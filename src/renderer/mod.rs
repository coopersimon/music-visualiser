pub mod circle;
pub mod quad;

use winit::window::Window;
use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashMap
};
use crate::audio::AudioPacket;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderableType {
    Circle,
    Quad
}

/// Provides the output display to the window.
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    pipelines: Rc<RefCell< HashMap<RenderableType, wgpu::RenderPipeline> >>
}

impl Renderer {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&Default::default());

        let adapter = futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })).expect("Failed to find appropriate adapter");

        let (device, queue) = futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            ..Default::default()
        })).expect("Failed to create device");

        let pipelines = Rc::new(RefCell::new(HashMap::new()));

        Self {
            instance,
            adapter,
            device,
            queue,

            pipelines
        }
    }

    pub fn create_surface(&self, window: std::sync::Arc<Window>) -> Surface {
        let size = window.inner_size();
        let surface = self.instance.create_surface(window.clone()).expect("Failed to create surface");
        let surface_config = surface.get_default_config(&self.adapter, size.width, size.height).expect("Could not get default surface config");
        surface.configure(&self.device, &surface_config);

        Surface {
            surface, surface_config
        }
    }

    pub fn resize_surface(&self, surface: &mut Surface, width: u32, height: u32) {
        surface.resize(width, height, &self.device);
    }
}

// Render commands
impl Renderer {
    pub fn new_render_pass<'a>(&'a self, surface: &mut Surface) -> RenderPass<'a> {
        let surface_tex = surface.surface.get_current_texture()
            .expect("Timeout when acquiring next swapchain tex.");
        let surface_tex_view = surface_tex.texture.create_view(&Default::default());
        let command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        RenderPass {
            renderer: &self,
            command_encoder,
            surface_tex,
            surface_tex_view,
            render_pass: None
        }
    }

    pub fn get_render_pipeline(&self, renderable: RenderableType) -> wgpu::RenderPipeline {
        let mut pipelines = self.pipelines.borrow_mut();

        if !pipelines.contains_key(&renderable) {
            let pipeline = match renderable {
                RenderableType::Circle =>   circle::CircleRenderable::create_pipeline(&self.device),
                RenderableType::Quad =>     quad::QuadRenderable::create_pipeline(&self.device),
            };
            pipelines.insert(renderable, pipeline);
        }

        pipelines.get(&renderable)
            .expect("could not find pipeline")
            .clone()
    }
}

pub struct RenderPass<'a> {
    renderer: &'a Renderer,
    command_encoder: wgpu::CommandEncoder,
    surface_tex: wgpu::SurfaceTexture,
    surface_tex_view: wgpu::TextureView,
    render_pass: Option<wgpu::RenderPass<'static>>
}

impl<'a> RenderPass<'a> {
    /// Begin the render pass and clear the framebuffer.
    pub fn begin(&mut self, clear_color: wgpu::Color) {
        let render_pass = self.command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.surface_tex_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        self.render_pass = Some(render_pass.forget_lifetime());
    }

    // TODO: move this to drop?
    /// End the render pass, submit to queue, and present.
    pub fn finish(mut self) {
        // End render pass.
        self.render_pass = None;
        let command_buffer = self.command_encoder.finish();
        self.renderer.queue.submit([command_buffer]);
        self.surface_tex.present();
    }
}

/// The surface connects the renderer to a window.
/// It is created with the Renderer.create_surface method.
pub struct Surface {
    surface:        wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

impl Surface {
    fn resize(&mut self, width: u32, height: u32, device: &wgpu::Device) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(device, &self.surface_config);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderParam {
    X,
    Y,
    R,
    G,
    B,
    Radius,
    LineWidth,
    Width,
    Height
}

pub trait Renderable {
    // TODO: store graphics params somewhere?
    /// Update the renderable with new parameters.
    fn update(&mut self, audio_packet: &AudioPacket, renderer: &Renderer, aspect_ratio: f32);

    /// Draw the renderable using the provided render pass.
    fn draw(&self, render_pass: &mut RenderPass<'_>);
}
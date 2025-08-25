pub mod object;
pub mod scene;

use winit::window::Window;
use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashMap
};
use crate::{
    audio::AudioPacket,
    operation::Operation
};
use scene::Scene;
use object::*;

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32
}

/// The overall image to render.
pub struct Display {
    scene: Box<dyn Scene>
}

impl Display {
    pub fn new(scene: Box<dyn Scene>) -> Self {
        Self {
            scene
        }
    }

    pub fn render(&mut self, renderer: &Renderer, audio_packet: &AudioPacket, surface: &mut Surface) {
        let size = Size { width: surface.surface_config.width, height: surface.surface_config.height };
        let surface_tex = surface.surface.get_current_texture().expect("could not get texture");

        self.scene.set_display(&surface_tex);
        self.scene.update(renderer, audio_packet, size);
        self.scene.draw(renderer);

        surface_tex.present();
    }
}

pub struct Mapping(HashMap<RenderParam, Operation>);

impl Mapping {
    pub fn new(from: (RenderParam, Operation)) -> Self {
        Self(
            HashMap::from([from])
        )
    }

    pub fn add(mut self, param: (RenderParam, Operation)) -> Self {
        self.0.insert(param.0, param.1);
        Self(self.0)
    }

    pub fn get(&mut self, param: RenderParam) -> Result<Operation, CreationError> {
        self.0.remove(&param).ok_or(CreationError::MissingParameter(param))
    }

    pub fn check_extra_parameters(&self) -> Result<(), CreationError> {
        if let Some((param, _)) = self.0.iter().next() {
            Err(CreationError::ExtraParameter(*param))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum CreationError {
    MissingParameter(RenderParam),
    ExtraParameter(RenderParam)
}

impl std::fmt::Display for CreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CreationError::*;
        match *self {
            MissingParameter(p) =>  write!(f, "missing required parameter {}", p),
            ExtraParameter(p) =>    write!(f, "invalid parameter {}", p),
        }
    }
}

/// Provides the output display to the window.
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    pipelines: Rc<RefCell< HashMap<ObjectType, wgpu::RenderPipeline> >>
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
    pub fn new_render_pass<'a>(&'a self, tex_view: wgpu::TextureView) -> RenderPass<'a> {
        let command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        RenderPass {
            renderer: &self,
            command_encoder,
            tex_view,
            render_pass: None
        }
    }

    pub fn get_render_pipeline(&self, renderable: ObjectType) -> wgpu::RenderPipeline {
        let mut pipelines = self.pipelines.borrow_mut();

        if !pipelines.contains_key(&renderable) {
            let pipeline = match renderable {
                ObjectType::Circle =>   circle::CircleRenderable::create_pipeline(&self.device),
                ObjectType::Quad =>     quad::QuadRenderable::create_pipeline(&self.device),
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
    command_encoder: wgpu::CommandEncoder, // TODO: this shouldn't make a new encoder every time.
    tex_view: wgpu::TextureView,
    render_pass: Option<wgpu::RenderPass<'static>>
}

impl<'a> RenderPass<'a> {
    /// Begin the render pass and clear the framebuffer.
    pub fn begin(&mut self, clear_color: wgpu::Color) {
        let render_pass = self.command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.tex_view,
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
    /// End the render pass and submit to queue.
    pub fn finish(mut self) {
        // End render pass.
        self.render_pass = None;
        let command_buffer = self.command_encoder.finish();
        self.renderer.queue.submit([command_buffer]);
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, strum::EnumString, strum::Display)]
pub enum RenderParam {
    #[strum(ascii_case_insensitive)]
    X,
    #[strum(ascii_case_insensitive)]
    Y,
    #[strum(ascii_case_insensitive)]
    R,
    #[strum(ascii_case_insensitive)]
    G,
    #[strum(ascii_case_insensitive)]
    B,
    #[strum(ascii_case_insensitive)]
    Radius,
    #[strum(serialize = "line_width")]
    LineWidth,
    #[strum(ascii_case_insensitive)]
    Width,
    #[strum(ascii_case_insensitive)]
    Height
}

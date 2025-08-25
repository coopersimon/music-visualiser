use super::object::ObjectRenderable;
use crate::{
    audio::AudioPacket,
    operation::Operation,
    renderer::{Renderer, Mapping, CreationError, RenderParam, Size}
};

/// A scene specifies an image that is created each frame.
/// It may output to the screen directly, or just write to a texture,
/// and later be used to compose the final image.
pub trait Scene {
    /// Set to use as the final display output.
    fn set_display(&mut self, surface: &wgpu::SurfaceTexture);

    fn update(&mut self, renderer: &Renderer, audio_packet: &AudioPacket, size: Size);

    fn draw(&mut self, renderer: &Renderer);

    // fn get_tex();
}

struct SceneTexture {
    size: Size,
    tex: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>
}

impl SceneTexture {
    fn new() -> Self {
        Self {
            size: Size { width: 0, height: 0 },
            tex: None,
            view: None
        }
    }

    fn update_size(&mut self, renderer: &Renderer, size: Size) {
        if self.size.width != size.width || self.size.height != size.height {
            let desc = wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: size.width, height: size.height, depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
            };
            self.tex = Some(renderer.device.create_texture(&desc));
            self.size = size;
            self.view = None;
        }
    }

    fn set_display(&mut self, surface: &wgpu::SurfaceTexture) {
        self.view = Some(surface.texture.create_view(&Default::default()));
    }

    fn get_view(&mut self) -> wgpu::TextureView {
        if self.view.is_none() && self.tex.is_some() {
            self.view = Some(self.tex.as_ref().unwrap().create_view(&Default::default()));
        }
        self.view.as_ref().unwrap().clone()
    }
}

pub struct RenderList {
    objects: Vec<Box<dyn ObjectRenderable>>,

    color: [f64; 3],
    r: Operation,
    g: Operation,
    b: Operation,

    tex: SceneTexture
}

impl RenderList {
    pub fn new(objects: Vec<Box<dyn ObjectRenderable>>, mut mapping: Mapping, _renderer: &Renderer) -> Result<Box<dyn Scene>, CreationError> {
        Ok(Box::new(Self {
            objects,

            color: [0.0, 0.0, 0.0],
            r: mapping.get(RenderParam::R)?,
            g: mapping.get(RenderParam::G)?,
            b: mapping.get(RenderParam::B)?,

            tex: SceneTexture::new()
        }))
    }
}

impl Scene for RenderList {
    fn set_display(&mut self, surface: &wgpu::SurfaceTexture) {
        self.tex.set_display(surface);
    }

    fn update(&mut self, renderer: &Renderer, audio_packet: &AudioPacket, size: Size) {
        self.tex.update_size(renderer, size);

        self.color = [
            self.r.eval(audio_packet).into(),
            self.g.eval(audio_packet).into(),
            self.b.eval(audio_packet).into()
        ];

        let aspect_ratio = (size.width as f32) / (size.height as f32);
        for object in &mut self.objects {
            object.update(renderer, audio_packet, aspect_ratio);
        }
    }

    fn draw(&mut self, renderer: &Renderer) {
        let view = self.tex.get_view();
        let mut render_pass = renderer.new_render_pass(view);
        render_pass.begin(wgpu::Color {
            r: self.color[0],
            g: self.color[1],
            b: self.color[2],
            a: 1.0
        });

        for object in &mut self.objects {
            object.draw(&mut render_pass);
        }

        render_pass.finish();
    }
}

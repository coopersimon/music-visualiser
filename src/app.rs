use winit::{
    application::ApplicationHandler, dpi::{
        LogicalSize, Size
    }, event::{
        WindowEvent
    }, window::Window
};

use crate::{
    audio::AudioSource,
    renderer::{Renderer, Scene, Surface}
};

/// State of the active window.
struct WindowState {
    window:  std::sync::Arc<Window>,
    surface: Surface,
}

/// Runtime state of the Application.
pub struct App {
    renderer: Renderer,
    audio_source: AudioSource,
    scene: Scene,
    window: Option<WindowState>,

    start_time: chrono::DateTime<chrono::Utc>,
}

impl App {
    pub fn new(renderer: Renderer, audio_source: AudioSource, scene: Scene) -> Self {
        Self {
            renderer,
            audio_source,
            scene,
            window: None,

            start_time: chrono::Utc::now()
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_inner_size(Size::Logical(LogicalSize{width: 1080.0, height: 720.0}))
            .with_title("Visualiser");
        let window = std::sync::Arc::new(event_loop.create_window(window_attrs).unwrap());

        let surface = self.renderer.create_surface(window.clone());

        self.window = Some(WindowState {
            window, surface
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(size) => {
                self.renderer.resize_surface(
                    &mut self.window.as_mut().unwrap().surface,
                    size.width, size.height);
            },
            WindowEvent::RedrawRequested => {
                let frame_time = chrono::Utc::now();
                let time = frame_time - self.start_time;

                let window_size = self.window.as_ref().unwrap().window.inner_size();
                let aspect_ratio = (window_size.width as f32) / (window_size.height as f32);

                let audio_packet = self.audio_source.get_frame_data(time.as_seconds_f32());

                let mut render_pass = self.renderer.new_render_pass(&mut self.window.as_mut().unwrap().surface);
                render_pass.begin(wgpu::Color::WHITE);

                for renderable in &mut self.scene.render_list {
                    renderable.update(&audio_packet, &self.renderer, aspect_ratio);
                    renderable.draw(&mut render_pass);
                }

                render_pass.finish();

                self.window.as_ref().unwrap().window.request_redraw();
            },
            _ => {},
        }
    }
}

use winit::{
    application::ApplicationHandler, dpi::{
        LogicalSize
    }, event::{
        WindowEvent
    }, window::Window
};

use crate::{
    audio::AudioSource,
    renderer::{Renderer, Display, Surface}
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
    display: Display,
    window: Option<WindowState>,

    start_time: chrono::DateTime<chrono::Utc>,
}

impl App {
    pub fn new(renderer: Renderer, audio_source: AudioSource, display: Display) -> Self {
        Self {
            renderer,
            audio_source,
            display,
            window: None,

            start_time: chrono::Utc::now()
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_inner_size(winit::dpi::Size::Logical(LogicalSize{width: 1080.0, height: 720.0}))
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

                let audio_packet = self.audio_source.get_frame_data(time.as_seconds_f32());

                self.display.render(&self.renderer, &audio_packet, &mut self.window.as_mut().unwrap().surface);

                self.window.as_ref().unwrap().window.request_redraw();
            },
            _ => {},
        }
    }
}

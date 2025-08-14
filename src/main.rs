mod app;
mod renderer;
mod audio;
mod operation;

use winit::event_loop::EventLoop;

use clap::Parser;

use operation::Mapping;
use audio::AudioParam;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    audio: String
}

fn main() {
    let args = Args::parse();

    let audio_source = audio::AudioSource::from_file(&args.audio);

    let audio_player = audio::AudioPlayer::new();
    audio_player.play(&audio_source);

    let renderer = renderer::Renderer::new();

    // TODO: create based on script.
    use renderer::RenderParam::*;
    use operation::Operation::*;
    let render_list: Vec<Box<dyn renderer::Renderable>> = vec![
        Box::new(renderer::circle::CircleRenderable::new(Mapping::from([
            (X, Const(0.0)),
            (Y, Const(0.0)),
            (Radius, Mul(Param(AudioParam::Amplitude).into(), Const(1.5).into())),
            (LineWidth, Const(0.01)),
            (R, Const(1.0)),
            (G, Const(0.0)),
            (B, Const(0.0))
        ]), &renderer))
    ];

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = app::App::new(renderer, audio_source, render_list);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

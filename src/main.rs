mod app;
mod renderer;
mod audio;
#[macro_use]
mod operation;
mod script;

use winit::event_loop::EventLoop;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    audio: String,
    #[arg(short, long)]
    script: String
}

fn main() {
    let args = Args::parse();

    let audio_source = audio::AudioSource::from_file(&args.audio);

    let audio_player = audio::AudioPlayer::new();
    audio_player.play(&audio_source);

    let renderer = renderer::Renderer::new();

    let scene = match script::parse_file(&args.script, &renderer) {
        Ok(s) => s,
        Err(e) => panic!("script error: {:?}", e)
    };

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = app::App::new(renderer, audio_source, scene);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

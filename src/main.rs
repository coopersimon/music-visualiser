mod app;
mod renderer;

use winit::event_loop::EventLoop;

use std::fs::File;
use rodio::{Decoder, OutputStream, source::Source};

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    audio: String
}

fn main() {
    let args = Args::parse();

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("could not open stream");
    let file = File::open(args.audio).expect("couldn't open audio file");
    let source = Decoder::try_from(file).unwrap();

    let channel_count = source.channels();

    let renderer = renderer::Renderer::new();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = app::App::new(renderer);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

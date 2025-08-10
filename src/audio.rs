
use std::fs::File;
use rodio::{Sink, Decoder, OutputStream, source::Source, source::Buffered};
use std::io::BufReader;

/// An audio source file.
pub struct AudioSource {
    buffer: Buffered<Decoder<BufReader<File>>>
}

impl AudioSource {
    // TODO: path arg should be a path type.
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).expect("couldn't open audio file");
        let source = Decoder::try_from(file).unwrap();

        //let channel_count = source.channels();
        let buffer = source.buffered();

        Self {
            buffer
        }
    }
}

/// Handles playback of the audio source to speakers.
pub struct AudioPlayer {
    output: OutputStream,
    sink: Sink
}

impl AudioPlayer {
    pub fn new() -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("could not find audio output");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        Self {
            output: stream_handle,
            sink
        }
    }

    pub fn play(&self, source: &AudioSource) {
        self.output.mixer().add(source.buffer.clone());
    }
}
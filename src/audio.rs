
use std::fs::File;
use rodio::{Sink, Decoder, OutputStream, source::Source, source::Buffered};
use std::io::BufReader;

/// An audio source file.
pub struct AudioSource {
    buffer: Buffered<Decoder<BufReader<File>>>,

    // TODO: internal buffers.
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

    pub fn get_frame_data(&mut self) -> AudioPacket {
        // TODO: pre-calc.
        let fps = self.buffer.sample_rate() / 60;
        let frame_size = (self.buffer.channels() as u32) * fps;

        let amplitude = self.buffer.by_ref()
            .take(frame_size as usize)
            .reduce(|acc, n| acc + n.abs())
            .unwrap() / (frame_size as f32);

        AudioPacket {
            amplitude
        }
    }
}

/// Audio data for a single frame.
pub struct AudioPacket {
    amplitude: f32
}

impl AudioPacket {
    pub fn get_param(&self, param: AudioParam) -> f32 {
        use AudioParam::*;
        match param {
            Amplitude => self.amplitude
        }
    }
}

#[derive(Clone, Copy)]
pub enum AudioParam {
    Amplitude
}

/*pub struct AudioParam {
    amplitude: f32
}*/

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
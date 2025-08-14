
use std::fs::File;
use rodio::{Sink, Decoder, OutputStream, source::Source, source::Buffered};
use std::io::BufReader;

const FRAME_SIZE: f32 = 0.5; // seconds

/// An audio source file.
pub struct AudioSource {
    buffer: Buffered<Decoder<BufReader<File>>>,
    frame_count: f32,
}

impl AudioSource {
    // TODO: path arg should be a path type.
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).expect("couldn't open audio file");
        let source = Decoder::try_from(file).unwrap();

        let buffer = source.buffered();
        let frame_count = (buffer.sample_rate() as f32) * FRAME_SIZE;

        Self {
            buffer,
            frame_count
        }
    }

    /// Get a frame of audio for a specified time in the song, defined in seconds.
    pub fn get_frame_data(&mut self, seconds: f32) -> AudioPacket {
        let start_time = seconds - FRAME_SIZE * 0.5;
        let frame_count = (self.frame_count - start_time.min(0.0)).round() as usize;
        let frame_start = ((self.buffer.sample_rate() as f32) * start_time.max(0.0)).round() as usize;

        let channel_count = self.buffer.channels() as usize;

        let buffer = self.buffer.clone()
            .skip(frame_start * channel_count)
            .take(frame_count * channel_count);
        let buffer_size = buffer.size_hint().1.unwrap_or(frame_count);

        let amplitude = buffer
            .reduce(|acc, n| acc + n.abs())
            .unwrap() / (buffer_size as f32);

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

/// Handles playback of the audio source to speakers.
pub struct AudioPlayer {
    output: OutputStream,
    _sink: Sink
}

impl AudioPlayer {
    pub fn new() -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("could not find audio output");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        Self {
            output: stream_handle,
            _sink: sink
        }
    }

    pub fn play(&self, source: &AudioSource) {
        self.output.mixer().add(source.buffer.clone());
    }
}
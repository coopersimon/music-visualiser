use std::collections::HashMap;
use crate::renderer::RenderParam;
use crate::audio::{AudioPacket, AudioParam};

pub type Mapping = HashMap<RenderParam, Operation>;

pub enum Operation {
    Const(f32),
    Param(AudioParam),
    Add(Box<Operation>, Box<Operation>),
    Sub(Box<Operation>, Box<Operation>),
    Mul(Box<Operation>, Box<Operation>),
    Div(Box<Operation>, Box<Operation>)
}

impl Operation {
    pub fn eval(&self, audio_packet: &AudioPacket) -> f32 {
        use Operation::*;
        match self {
            Const(n) => *n,
            Param(p) => audio_packet.get_param(*p),
            Add(a, b) => a.eval(audio_packet) + b.eval(audio_packet),
            Sub(a, b) => a.eval(audio_packet) - b.eval(audio_packet),
            Mul(a, b) => a.eval(audio_packet) * b.eval(audio_packet),
            Div(a, b) => a.eval(audio_packet) / b.eval(audio_packet),
        }
    }
}
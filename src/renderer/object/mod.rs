pub mod circle;
pub mod quad;

use crate::audio::AudioPacket;
use crate::renderer::{Renderer, RenderPass};

pub trait ObjectRenderable {
    // TODO: store graphics params somewhere?
    /// Update the renderable with new parameters.
    fn update(&mut self, renderer: &Renderer, audio_packet: &AudioPacket, aspect_ratio: f32);

    /// Draw the renderable using the provided render pass.
    fn draw(&self, render_pass: &mut RenderPass<'_>);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
pub enum ObjectType {
    #[strum(ascii_case_insensitive)]
    Circle,
    #[strum(ascii_case_insensitive)]
    Quad
}
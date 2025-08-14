use bytemuck::{Zeroable, Pod};
use wgpu::util::DeviceExt;

use super::{Renderer, RenderPass, Renderable, RenderParam};
use crate::{
    operation::Mapping,
    audio::AudioPacket
};

const CIRCLE_SIZE: usize = 90;
const VERTEX_COUNT: usize = (CIRCLE_SIZE + 1) * 2;

/// An instance of a circle.
pub struct CircleRenderable {
    mapping: Mapping,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup
}

impl CircleRenderable {
    pub fn create_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None
                }
            ]
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders/circle.wgsl"));
        let circle_desc = wgpu::RenderPipelineDescriptor {
            label: Some("circle"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0
                            }
                        ]
                    }
                ],
                compilation_options: Default::default()
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                .. Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default()
            }),
            multiview: None,
            cache: None
        };
        device.create_render_pipeline(&circle_desc)
    }

    /// Create a new circle to display on-screen.
    pub fn new(mapping: Mapping, renderer: &Renderer) -> Self {
        // TODO: share vertex buffer?
        let mut buf = Vec::new();
        for step in 0..=CIRCLE_SIZE {
            let radians = (step as f32) / (CIRCLE_SIZE as f32) * (2.0 * std::f32::consts::PI);
            let x = radians.sin();
            let y = radians.cos();
            buf.push(Vertex{pos: [x, y]}); // Inner circle vertex
            buf.push(Vertex{pos: [x, y]}); // Outer circle vertex
        }
        let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::VERTEX,
            contents: &bytemuck::cast_slice(&buf)
        });
        let uniform_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<f32>() as u64) * 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });
        let pipeline = renderer.get_render_pipeline(super::RenderableType::Circle);
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding()
                }
            ]
        });
        Self {
            mapping,
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group
        }
    }
}

impl Renderable for CircleRenderable {
    /// Update the circle with new parameters.
    fn update(&mut self, audio_packet: &AudioPacket, renderer: &Renderer, aspect_ratio: f32) {
        let uniform_data = [
            aspect_ratio,
            self.mapping[&RenderParam::X].eval(audio_packet),
            self.mapping[&RenderParam::Y].eval(audio_packet),
            self.mapping[&RenderParam::Radius].eval(audio_packet),
            self.mapping[&RenderParam::LineWidth].eval(audio_packet),
            self.mapping[&RenderParam::R].eval(audio_packet),
            self.mapping[&RenderParam::G].eval(audio_packet),
            self.mapping[&RenderParam::B].eval(audio_packet)
        ];
        renderer.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&uniform_data));
    }

    /// Draw the circle using the provided render pass.
    fn draw(&self, render_pass: &mut RenderPass<'_>) {
        let render_pass = render_pass.render_pass.as_mut().unwrap();
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        let max = VERTEX_COUNT as u32;
        render_pass.draw(0..max, 0..1);
    }
}

#[derive(Zeroable, Pod, Clone, Copy)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2]
}

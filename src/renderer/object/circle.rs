use bytemuck::{Zeroable, Pod};
use wgpu::util::DeviceExt;

use crate::{
    audio::AudioPacket, operation::Operation, renderer::{
        Renderer, RenderPass, RenderParam, Mapping, CreationError
    }
};
use super::{ObjectRenderable, ObjectType};

const CIRCLE_SIZE: usize = 90;
const VERTEX_COUNT: usize = (CIRCLE_SIZE + 1) * 2;

#[derive(Zeroable, Pod, Clone, Copy)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2]
}

/// An instance of a circle.
pub struct CircleRenderable {
    params: CircleParameters,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup
}

struct CircleParameters {
    x: Operation,
    y: Operation,
    radius: Operation,
    line_width: Operation,
    r: Operation,
    g: Operation,
    b: Operation,
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
    pub fn new(mut mapping: Mapping, renderer: &Renderer) -> Result<Self, CreationError> {
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
        let pipeline = renderer.get_render_pipeline(ObjectType::Circle);
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
        let params = CircleParameters {
            x: mapping.get(RenderParam::X)?,
            y: mapping.get(RenderParam::Y)?,
            radius: mapping.get(RenderParam::Radius)?,
            line_width: mapping.get(RenderParam::LineWidth)?,
            r: mapping.get(RenderParam::R)?,
            g: mapping.get(RenderParam::G)?,
            b: mapping.get(RenderParam::B)?,
        };
        mapping.check_extra_parameters()?;
        Ok(Self {
            params,
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group
        })
    }
}

impl ObjectRenderable for CircleRenderable {
    fn update(&mut self, renderer: &Renderer, audio_packet: &AudioPacket, aspect_ratio: f32) {
        let uniform_data = [
            aspect_ratio,
            self.params.x.eval(audio_packet),
            self.params.y.eval(audio_packet),
            self.params.radius.eval(audio_packet),
            self.params.line_width.eval(audio_packet),
            self.params.r.eval(audio_packet),
            self.params.g.eval(audio_packet),
            self.params.b.eval(audio_packet)
        ];
        renderer.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&uniform_data));
    }

    fn draw(&self, render_pass: &mut RenderPass<'_>) {
        let render_pass = render_pass.render_pass.as_mut().unwrap();
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        let max = VERTEX_COUNT as u32;
        render_pass.draw(0..max, 0..1);
    }
}

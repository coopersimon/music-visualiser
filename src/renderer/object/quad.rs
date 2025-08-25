use bytemuck::{Zeroable, Pod};
use wgpu::util::DeviceExt;

use crate::{
    audio::AudioPacket, operation::Operation, renderer::{
        Renderer, RenderPass, RenderParam, Mapping, CreationError
    }
};
use super::{ObjectRenderable, ObjectType};

const VERTEX_COUNT: usize = 4;

#[derive(Zeroable, Pod, Clone, Copy)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2]
}

/// An instance of a quad.
pub struct QuadRenderable {
    params: QuadParameters,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup
}

struct QuadParameters {
    x: Operation,
    y: Operation,
    width: Operation,
    height: Operation,
    r: Operation,
    g: Operation,
    b: Operation,
}

impl QuadRenderable {
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
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shaders/quad.wgsl"));
        let circle_desc = wgpu::RenderPipelineDescriptor {
            label: Some("quad"),
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

    /// Create a new quad to display on-screen.
    pub fn new(mut mapping: Mapping, renderer: &Renderer) -> Result<Self, CreationError> {
        // TODO: share vertex buffer?
        let buf = [
            Vertex{pos: [0.0, 0.0]},
            Vertex{pos: [1.0, 0.0]},
            Vertex{pos: [0.0, 1.0]},
            Vertex{pos: [1.0, 1.0]}
        ];
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
        let pipeline = renderer.get_render_pipeline(ObjectType::Quad);
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
        let params = QuadParameters {
            x: mapping.get(RenderParam::X)?,
            y: mapping.get(RenderParam::Y)?,
            width: mapping.get(RenderParam::Width)?,
            height: mapping.get(RenderParam::Height)?,
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

impl ObjectRenderable for QuadRenderable {
    fn update(&mut self, renderer: &Renderer, audio_packet: &AudioPacket, _aspect_ratio: f32) {
        let uniform_data = [
            self.params.x.eval(audio_packet),
            self.params.y.eval(audio_packet),
            self.params.width.eval(audio_packet),
            self.params.height.eval(audio_packet),
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

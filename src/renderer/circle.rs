use bytemuck::{Zeroable, Pod};

use super::{Renderer, RenderPass, Renderable};

const CIRCLE_SIZE: usize = 90;

/// An instance of a circle.
pub struct CircleRenderable {
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                        array_stride: (std::mem::size_of::<f32>() as u64) * 2,
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
                topology: wgpu::PrimitiveTopology::LineStrip,
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
    pub fn new(renderer: &Renderer) -> Self {
        let vertex_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<Vertex>() * (CIRCLE_SIZE + 1)) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });
        let uniform_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<f32>() as u64) * 4,
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
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group
        }
    }
}

impl Renderable for CircleRenderable {
    type Params = CircleParams;

    /// Update the circle with new parameters.
    fn update(&mut self, params: &Self::Params, renderer: &Renderer) {
        // TODO: compute shader to generate vertices?

        // Update vertex buffer.
        // TODO: don't realloc every time.
        let mut buf = Vec::new();
        for step in 0..=CIRCLE_SIZE {
            let radians = (step as f32) / (CIRCLE_SIZE as f32) * (2.0 * std::f32::consts::PI);
            let x = params.x_pos + params.radius * radians.sin();
            let y = params.y_pos + params.radius * radians.cos() * params.aspect_ratio;
            buf.push(Vertex{pos: [x, y]});
        }
        renderer.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&buf));

        // Update binding data
        let color_buffer: [f32; 4] = [params.color[0], params.color[1], params.color[2], 1.0];
        renderer.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&color_buffer));
    }

    /// Draw the circle using the provided render pass.
    fn draw(&self, render_pass: &mut RenderPass<'_>) {
        let render_pass = render_pass.render_pass.as_mut().unwrap();
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        let max = (CIRCLE_SIZE + 1) as u32;
        render_pass.draw(0..max, 0..1);
    }
}

pub struct CircleParams {
    pub aspect_ratio: f32,

    pub x_pos: f32,
    pub y_pos: f32,
    pub radius: f32,
    //pub line_thickness: f32,
    pub color: [f32; 3]
}

#[derive(Zeroable, Pod, Clone, Copy)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2]
}

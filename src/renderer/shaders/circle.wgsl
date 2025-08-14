struct VertexInput {
    @location(0) pos: vec2<f32>,
    @builtin(vertex_index) index: u32
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>
}

struct Uniforms {
    aspect_ratio: f32, // TODO: move this elsewhere?

    x: f32,
    y: f32,
    radius: f32,
    line_width: f32,
    r: f32,
    g: f32,
    b: f32
}

@group(0) @binding(0) var<uniform> params: Uniforms;

@vertex fn vs_main(
    vertex: VertexInput
) -> VertexOutput {
    let outer_circle = vertex.index % 2 == 1;
    let radius = params.radius + select(params.line_width, -params.line_width, outer_circle);
    var pos = vertex.pos * radius;
    pos.y *= params.aspect_ratio;
    pos += vec2<f32>(params.x, params.y);
    var out: VertexOutput;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(params.r, params.g, params.b, 1.0);
}
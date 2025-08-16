struct VertexInput {
    @location(0) pos: vec2<f32>
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>
}

struct Uniforms {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    r: f32,
    g: f32,
    b: f32
}

@group(0) @binding(0) var<uniform> params: Uniforms;

@vertex fn vs_main(
    vertex: VertexInput
) -> VertexOutput {
    let x = params.x + vertex.pos.x * params.width;
    let y = params.y + vertex.pos.y * params.height;
    var out: VertexOutput;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(params.r, params.g, params.b, 1.0);
}
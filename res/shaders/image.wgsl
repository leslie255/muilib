@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;
@group(0) @binding(1) var<uniform> aaf: f32;

@group(1) @binding(0) var<uniform> model_view: mat4x4<f32>;
@group(1) @binding(1) var texture: texture_2d<f32>;
@group(1) @binding(2) var sampler_: sampler;

const vertices = array<vec2<f32>, 6>(
    vec2<f32>(0., 0.),
    vec2<f32>(1., 0.),
    vec2<f32>(1., 1.),
    vec2<f32>(0., 0.),
    vec2<f32>(1., 1.),
    vec2<f32>(0., 1.),
);

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var result: VertexOutput;
    let position = vertices[index];
    // result.uv = vec2<f32>(position.x, 1. - position.y);
    result.uv = position;
    result.position = projection * model_view * vec4<f32>(position.xy, 0.0, 1.0);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sampler_, vertex.uv);
}

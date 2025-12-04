@group(0) @binding(0) var<uniform> model_view: mat4x4<f32>;
@group(0) @binding(1) var<uniform> projection: mat4x4<f32>;
@group(0) @binding(2) var<uniform> fill_color: vec4<f32>;
@group(0) @binding(3) var<uniform> line_color: vec4<f32>;
@group(0) @binding(4) var<uniform> line_width: vec2<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var result: VertexOutput;
    result.uv = input.uv;
    result.position = projection * model_view * vec4<f32>(input.position.xy, 0.0, 1.0);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // Distance to border.
    let distance_x = min(vertex.uv.x, 1.0 - vertex.uv.x);
    let distance_y = min(vertex.uv.y, 1.0 - vertex.uv.y);
    if distance_x <= line_width.x || distance_y <= line_width.y {
        return line_color;
    } else {
        return fill_color;
    }
}

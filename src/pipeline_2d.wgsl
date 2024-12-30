struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var sim_texture: texture_2d<f32>;
@group(1) @binding(1)
var sim_sampler: sampler;


//struct VertexInput {
//    @location(0) position: vec3<f32>,
//    @location(1) color: vec3<f32>,
//}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

//@vertex
//fn vs_main(
//    model: VertexInput,
//) -> VertexOutput {
//    var out: VertexOutput;
////    out.color = model.color;
//    out.color = vec3<f32>(model.position);
//    out.tex_coords = vec2<f32>(model.position.xy);
//    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
//    return out;
//}
@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
     var color = textureSample(sim_texture, sim_sampler, in.uv);
     return vec4<f32>(color.r, color.g, 0.0, 1.0);
}

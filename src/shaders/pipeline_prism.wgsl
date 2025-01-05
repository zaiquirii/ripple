struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var sim_texture: texture_2d<f32>;
@group(1) @binding(1)
var sim_sampler: sampler;

struct VertexInput {
    @location(0) vertex: vec3<f32>,
    @location(1) position: vec2<f32>,
    @location(2) sim_coord: vec2<u32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var sim_cell = textureLoad(sim_texture, model.sim_coord, 0);
    var position = model.vertex;
    position.x += model.position.x;
    position.z += model.position.y;
    position.y += sim_cell.r * 15.0;


    var out: VertexOutput;
    out.color = model.vertex;
    out.world_pos = position;
    out.world_normal = model.normal;
    out.tex_coords= vec2<f32>(f32(model.sim_coord.x) / 128.0, f32(model.sim_coord.y) / 128.0);
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    return out;
}

// Fragment shader
const light_pos = vec3<f32>(0.0, 2.0, 0.0);
const light_color = vec3<f32>(1.0, 1.0, 1.0);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength = 0.1;
    let ambient_color = light_color * ambient_strength;

    let light_dir = normalize(light_pos - in.world_pos);
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light_color * diffuse_strength;

    var tex_color = textureSample(sim_texture, sim_sampler, in.tex_coords);
    var sim_color = vec3<f32>(tex_color.r * 2.0, 0.0, tex_color.r * 2.0);
    sim_color += 0.001;

    let result = (ambient_color + diffuse_color) * sim_color;
    return vec4<f32>(result, 1.0);
}

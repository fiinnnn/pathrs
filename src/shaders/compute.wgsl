[[group(0), binding(0)]]
var t_output: [[access(write)]] texture_storage_2d<rgba32float>;

[[block]]
struct CameraUniforms {
    width: u32;
    height: u32;
};

[[group(0), binding(1)]]
var<uniform> camera_uniforms: CameraUniforms;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(workgroup_id)]] coord: vec3<u32>) {
    let col = normalize(vec3<f32>(abs(-1.0 + (f32(coord.x)/(f32(camera_uniforms.width)/2.0))), abs(-1.0 + (f32(coord.y)/(f32(camera_uniforms.height)/2.0))), 1.0));
    textureStore(t_output, vec2<i32>(coord.xy), vec4<f32>(col, 1.0));
}
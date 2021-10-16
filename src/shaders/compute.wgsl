[[group(0), binding(0)]]
var t_output: texture_storage_2d<rgba32float, write>;

[[block]]
struct CameraUniforms {
    width: u32;
    height: u32;
};

[[group(0), binding(1)]]
var<uniform> camera_uniforms: CameraUniforms;

fn intersect_sphere(o: vec3<f32>, d: vec3<f32>) -> bool {
    let sp = vec3<f32>(0.0, 0.0, -5.0);
    let sr = 1.0;
    let m = o - sp;
    let b = dot(m, d);
    let c = dot(m, m) - sr * sr;

    if (c > 0.0 && b > 0.0) { return false; }

    let d = b * b - c;

    return (d >= 0.0);
}

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(workgroup_id)]] coord: vec3<u32>) {
    let ar = f32(camera_uniforms.width) / f32(camera_uniforms.height);
    let dir = normalize(vec3<f32>((-1.0 + (f32(coord.x)/(f32(camera_uniforms.width)/2.0))) * ar, -1.0 + (f32(coord.y)/(f32(camera_uniforms.height)/2.0)), -1.0));
    let origin = vec3<f32>(0.0, 0.0, 0.0);

    var col = vec3<f32>(0.0, 0.0, 0.0);
    if (intersect_sphere(origin, dir)) {
        col = vec3<f32>(1.0, 1.0, 1.0);
    }
    textureStore(t_output, vec2<i32>(coord.xy), vec4<f32>(col, 1.0));
}

[[group(0), binding(0)]]
var t_output: texture_storage_2d<rgba32float, write>;

struct CameraUniforms {
    pos: vec4<f32>;
    p0: vec4<f32>;
    p0p1: vec4<f32>;
    p0p2: vec4<f32>;
    width: f32;
    height: f32;
};

[[group(0), binding(1)]]
var<uniform> camera_uniforms: CameraUniforms;

fn intersect_triangle(o: vec3<f32>, d: vec3<f32>) -> f32 {
    let v0 = vec3<f32>(0.0, 1.0, -1.0);
    let v1 = vec3<f32>(-1.0, -0.5, -1.0);
    let v2 = vec3<f32>(1.0, -0.5, -1.0);

    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let T = o - v0;
    let p = cross(d, e2);
    let f = dot(p, e1);

    let u = dot(p, T) / f;
    if (u < -0.0001 || u > 1.0) { return 100.0; }

    let q = cross(T, e1);
    let v = dot(q, d) / f;
    if (v < -0.0001 || u+v > 1.0) { return 100.0; }

    return dot(q, e2) / f;
}

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(workgroup_id)]] coord: vec3<u32>) {
    let origin = camera_uniforms.pos.xyz;

    let screen_pos = camera_uniforms.p0 + camera_uniforms.p0p1 * (f32(coord.x) / camera_uniforms.width) 
      + camera_uniforms.p0p2 * ((camera_uniforms.height - f32(coord.y)) / camera_uniforms.height);

    let dir = normalize(screen_pos.xyz - origin);

    var col = vec3<f32>(1.0 - intersect_triangle(origin, dir) / 4.0);

    textureStore(t_output, vec2<i32>(coord.xy), vec4<f32>(col, 1.0));
}

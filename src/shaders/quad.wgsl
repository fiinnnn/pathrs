[[stage(vertex)]]
fn main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let x = -1.0 + f32((vertex_index & 1u) << 2u);
    let y = -1.0 + f32((vertex_index & 2u) << 1u);
    return vec4<f32>(x, y, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_output: texture_storage_2d<rgba32float, read>;

[[stage(fragment)]]
fn main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {
    return textureLoad(t_output, vec2<i32>(coord_in.xy));
}

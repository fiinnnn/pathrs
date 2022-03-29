#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

extern crate spirv_std;

use spirv_std::{
    glam::{UVec3, Vec3Swizzles, Vec4},
    Image,
};

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[spirv(compute(threads(1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] out_texture: &Image!(2D, format=rgba8, sampled=false),
) {
    unsafe {
        out_texture.write(
            id.xy(),
            Vec4::new(id.x as f32 / 1280.0, id.y as f32 / 720.0, 0.0, 1.0),
        );
    }
}

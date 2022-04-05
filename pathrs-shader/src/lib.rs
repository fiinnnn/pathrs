#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

extern crate spirv_std;

use pathrs_shared::Camera;
use spirv_std::{
    glam::{UVec3, Vec3Swizzles, Vec4},
    Image,
};

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[spirv(compute(threads(1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] out_texture: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] camera: &mut Camera,
) {
    // generate primary ray

    unsafe {
        out_texture.write(
            id.xy(),
            Vec4::new(
                id.x as f32 / camera.width,
                id.y as f32 / camera.height,
                camera.position.z,
                1.0,
            ),
        );
    }
}

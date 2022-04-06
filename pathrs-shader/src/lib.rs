#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

extern crate spirv_std;

use pathrs_shared::Viewport;
use spirv_std::{
    glam::{UVec3, Vec3, Vec3Swizzles, Vec4Swizzles},
    Image,
};

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[spirv(compute(threads(1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] out_texture: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] viewport: &mut Viewport,
) {
    // generate primary ray
    let u = id.x as f32 / viewport.width;
    let v = id.y as f32 / viewport.height;

    let dir =
        viewport.lower_left.xyz() + u * viewport.horizontal.xyz() + v * viewport.vertical.xyz()
            - viewport.origin.xyz();

    let t = 0.7 * (dir.normalize().y + 1.0);
    let col = (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0);

    unsafe {
        out_texture.write(id.xy(), col.extend(1.0));
    }
}

#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

extern crate spirv_std;

use pathrs_shared::Viewport;
use spirv_std::{
    num_traits::Float,
    glam::{UVec3, Vec3, Vec3Swizzles, Vec4Swizzles},
    Image,
};

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[spirv(compute(threads(16, 16)))]
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

    let col = if intersect(Vec3::new(0.0, 0.0, 0.0), dir).is_some() {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        let t = 0.7 * (dir.normalize().y + 1.0);
        (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
    };

    unsafe {
        // out_texture.write(id.xy(), Vec3::new(dir.x.abs(), dir.y.abs(), 0.0).extend(1.0));
        out_texture.write(id.xy(), col.extend(1.0));
    }
}

fn intersect(origin: Vec3, dir: Vec3) -> Option<f32> {
    let center = Vec3::new(0.0, 0.0, -5.0);
    let radius = 1.0;

    let oc = origin - center;
    let a = dir.dot(dir);
    let b = oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let d = b * b - a * c;

    if d > 0.0 {
        let t = (-b - d.sqrt()) / a;
        if t > 0.001 && t < 1000.0 {
            return Some(t);
        }
        let t = (-b + d.sqrt()) / a;
        if t > 0.001 && t < 1000.0 {
            return Some(t);
        }
    }
    None
}

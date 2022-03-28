#![no_std]

use spirv_std::glam::Vec3;

#[repr(C)]
pub struct Ray {
    pub pos: Vec3,
    pub dir: Vec3,
}

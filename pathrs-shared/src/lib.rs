#![no_std]

#[cfg(not(target_arch = "spirv"))]
use bytemuck::{Pod, Zeroable};

use spirv_std::glam::{Vec3, Vec4};

#[cfg_attr(not(target_arch = "spirv"), derive(Copy, Clone, Pod, Zeroable))]
#[repr(C)]
pub struct Viewport {
    pub origin: Vec4,
    pub lower_left: Vec4,
    pub horizontal: Vec4,
    pub vertical: Vec4,
    pub width: f32,
    pub height: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Viewport {
    pub fn new(
        origin: Vec3,
        lower_left: Vec3,
        horizontal: Vec3,
        vertical: Vec3,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            origin: origin.extend(0.0),
            lower_left: lower_left.extend(0.0),
            horizontal: horizontal.extend(0.0),
            vertical: vertical.extend(0.0),
            width,
            height,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

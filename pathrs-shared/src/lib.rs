#![no_std]

#[cfg(not(target_arch = "spirv"))]
use bytemuck::{Pod, Zeroable};

use spirv_std::glam::{vec4, Vec4};

#[derive(Clone, Copy, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Pod, Zeroable))]
#[repr(C)]
pub struct Camera {
    pub position: Vec4,
    // pub forward: Vec3,
    // pub up: Vec3,
    // pub right: Vec3,
    pub width: f32,
    pub height: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: vec4(0.0, 0.0, 0.0, 0.0),
            width,
            height,
            ..Default::default()
        }
    }
}

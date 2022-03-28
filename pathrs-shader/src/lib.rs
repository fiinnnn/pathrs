#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

extern crate spirv_std;

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[spirv(compute(threads(1)))]
pub fn main_cs() {}

use std::arch::x86_64::*;

use glam::{Vec3, Vec3A};

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub union f32x8 {
    pub lanes: [f32; 8],
    pub m256: __m256,
}

impl std::fmt::Debug for f32x8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("f32x8")
            .field(unsafe { &self.lanes })
            .finish()
    }
}

impl Default for f32x8 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl f32x8 {
    pub const ZERO: f32x8 = f32x8 { lanes: [0.0; 8] };
    pub const ONE: f32x8 = f32x8 { lanes: [1.0; 8] };

    pub const EPSILON: f32x8 = f32x8 {
        lanes: [f32::EPSILON; 8],
    };
    pub const NEGATIVE_EPSILON: f32x8 = f32x8 {
        lanes: [-f32::EPSILON; 8],
    };

    #[inline(always)]
    pub fn splat(v: f32) -> f32x8 {
        f32x8 { lanes: [v; 8] }
    }

    #[inline(always)]
    pub fn from_array(lanes: [f32; 8]) -> f32x8 {
        f32x8 { lanes }
    }

    #[inline(always)]
    pub fn fmadd(a: f32x8, b: f32x8, c: f32x8) -> f32x8 {
        unsafe { _mm256_fmadd_ps(a.m256, b.m256, c.m256) }.into()
    }

    #[inline(always)]
    pub fn fmsub(a: f32x8, b: f32x8, c: f32x8) -> f32x8 {
        unsafe { _mm256_fmsub_ps(a.m256, b.m256, c.m256) }.into()
    }

    #[inline(always)]
    pub fn sqrt(self) -> f32x8 {
        unsafe { _mm256_sqrt_ps(self.m256) }.into()
    }

    #[inline(always)]
    pub fn cmp_lt(self, rhs: f32x8) -> Bitmask {
        Bitmask {
            m256: unsafe { _mm256_cmp_ps::<_CMP_LT_OQ>(self.m256, rhs.m256) },
        }
    }

    #[inline(always)]
    pub fn cmp_gt(self, rhs: f32x8) -> Bitmask {
        Bitmask {
            m256: unsafe { _mm256_cmp_ps::<_CMP_GT_OQ>(self.m256, rhs.m256) },
        }
    }

    #[inline(always)]
    pub fn cmp_eq(self, rhs: f32x8) -> Bitmask {
        Bitmask {
            m256: unsafe { _mm256_cmp_ps::<_CMP_EQ_OQ>(self.m256, rhs.m256) },
        }
    }

    #[inline(always)]
    pub fn min(a: f32x8, b: f32x8) -> f32x8 {
        unsafe { _mm256_min_ps(a.m256, b.m256) }.into()
    }

    #[inline(always)]
    pub fn blend(a: f32x8, b: f32x8, mask: Bitmask) -> f32x8 {
        unsafe { _mm256_blendv_ps(a.m256, b.m256, mask.m256) }.into()
    }

    #[inline(always)]
    pub fn permute<const IMM8: i32>(self) -> f32x8 {
        unsafe { _mm256_permute_ps::<IMM8>(self.m256) }.into()
    }

    #[inline(always)]
    pub fn permute_2f128<const IMM8: i32>(a: f32x8, b: f32x8) -> f32x8 {
        unsafe { _mm256_permute2f128_ps::<IMM8>(a.m256, b.m256) }.into()
    }

    #[inline(always)]
    pub fn movemask(self) -> i32 {
        unsafe { _mm256_movemask_ps(self.m256) }
    }
}

impl std::ops::Mul for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_mul_ps(self.m256, rhs.m256) }.into()
    }
}

impl std::ops::Div for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_div_ps(self.m256, rhs.m256) }.into()
    }
}

impl std::ops::Add for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_add_ps(self.m256, rhs.m256) }.into()
    }
}

impl std::ops::Sub for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_sub_ps(self.m256, rhs.m256) }.into()
    }
}

impl std::ops::Index<usize> for f32x8 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &self.lanes[index] }
    }
}

impl From<__m256> for f32x8 {
    #[inline(always)]
    fn from(value: __m256) -> Self {
        Self { m256: value }
    }
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub union i32x8 {
    pub lanes: [i32; 8],
    pub m256i: __m256i,
}

impl std::fmt::Debug for i32x8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("i32x8")
            .field(unsafe { &self.lanes })
            .finish()
    }
}

impl i32x8 {
    #[inline(always)]
    pub fn splat(v: i32) -> i32x8 {
        i32x8 { lanes: [v; 8] }
    }

    #[inline(always)]
    pub fn from_array(lanes: [i32; 8]) -> Self {
        Self { lanes }
    }

    #[inline(always)]
    pub fn select(a: i32x8, b: i32x8, mask: Bitmask) -> i32x8 {
        unsafe { _mm256_blendv_epi8(a.m256i, b.m256i, mask.m256i) }.into()
    }

    #[inline(always)]
    pub fn transmute_f32x8(self) -> f32x8 {
        unsafe { std::mem::transmute(self) }
    }
}

impl std::ops::Add for i32x8 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_add_epi32(self.m256i, rhs.m256i) }.into()
    }
}

impl std::ops::AddAssign for i32x8 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl std::ops::Index<usize> for i32x8 {
    type Output = i32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &self.lanes[index] }
    }
}

impl From<__m256i> for i32x8 {
    #[inline(always)]
    fn from(value: __m256i) -> Self {
        Self { m256i: value }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union Bitmask {
    pub m256: __m256,
    pub m256i: __m256i,
}

impl Bitmask {
    #[inline(always)]
    pub fn as_f32x8(self) -> f32x8 {
        f32x8 {
            m256: unsafe { self.m256 },
        }
    }
}

impl From<__m256> for Bitmask {
    #[inline(always)]
    fn from(value: __m256) -> Self {
        Self { m256: value }
    }
}

impl From<__m256i> for Bitmask {
    #[inline(always)]
    fn from(value: __m256i) -> Self {
        Self { m256i: value }
    }
}

impl std::ops::BitAnd for Bitmask {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_and_ps(self.m256, rhs.m256) }.into()
    }
}

impl std::ops::BitAndAssign for Bitmask {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOr for Bitmask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { _mm256_or_ps(self.m256, rhs.m256) }.into()
    }
}

impl std::ops::BitOrAssign for Bitmask {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vec3x8 {
    pub x: f32x8,
    pub y: f32x8,
    pub z: f32x8,
}

impl From<Vec3> for Vec3x8 {
    #[inline(always)]
    fn from(value: Vec3) -> Self {
        Self {
            x: f32x8::splat(value.x),
            y: f32x8::splat(value.y),
            z: f32x8::splat(value.z),
        }
    }
}

impl From<Vec3A> for Vec3x8 {
    #[inline(always)]
    fn from(value: Vec3A) -> Self {
        Self {
            x: f32x8::splat(value.x),
            y: f32x8::splat(value.y),
            z: f32x8::splat(value.z),
        }
    }
}

impl Vec3x8 {
    #[inline(always)]
    pub fn dot(self, rhs: Vec3x8) -> f32x8 {
        f32x8::fmadd(self.z, rhs.z, f32x8::fmadd(self.y, rhs.y, self.x * rhs.x))
    }

    #[inline(always)]
    pub fn cross(self, rhs: Vec3x8) -> Vec3x8 {
        Vec3x8 {
            x: f32x8::fmsub(self.y, rhs.z, rhs.y * self.z),
            y: f32x8::fmsub(self.z, rhs.x, rhs.z * self.x),
            z: f32x8::fmsub(self.x, rhs.y, rhs.x * self.y),
        }
    }
}

impl std::ops::Sub for Vec3x8 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

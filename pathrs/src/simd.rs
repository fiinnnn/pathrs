use std::arch::x86_64::*;

use bevy::math::Vec3A;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub union f32x8 {
    f32: [f32; 8],
    m256: __m256,
}

impl f32x8 {
    const ZERO: f32x8 = f32x8 { f32: [0.0; 8] };
    const EPSILON: f32x8 = f32x8 { f32: [f32::EPSILON; 8] };
    const NEGATIVE_EPSILON: f32x8 = f32x8 { f32: [-f32::EPSILON; 8] };
}

impl From<f32> for f32x8 {
    fn from(value: f32) -> Self {
        f32x8 { f32: [value; 8] }
    }
}

#[cfg(feature = "simd")]
#[inline(always)]
fn dot(a: [f32x8; 3], b: [f32x8; 3]) -> f32x8 {
    f32x8 {
        m256: unsafe {
            _mm256_fmadd_ps(
                a[2].m256,
                b[2].m256,
                _mm256_fmadd_ps(a[1].m256, b[1].m256, _mm256_mul_ps(a[0].m256, b[0].m256)),
            )
        },
    }
}

#[cfg(feature = "simd")]
#[inline(always)]
fn cross(a: [f32x8; 3], b: [f32x8; 3], mut res: [f32x8; 3]) {
    res[0] = f32x8 {
        m256: unsafe { _mm256_fmsub_ps(a[1].m256, b[2].m256, _mm256_mul_ps(b[1].m256, a[2].m256)) },
    };
    res[1] = f32x8 {
        m256: unsafe { _mm256_fmsub_ps(a[2].m256, b[0].m256, _mm256_mul_ps(b[2].m256, a[0].m256)) },
    };
    res[2] = f32x8 {
        m256: unsafe { _mm256_fmsub_ps(a[0].m256, b[1].m256, _mm256_mul_ps(b[0].m256, a[1].m256)) },
    };
}

#[cfg(feature = "simd")]
#[inline(always)]
fn sub(a: [f32x8; 3], b: [f32x8; 3]) -> [f32x8; 3] {
    unsafe {
        [
            f32x8 { m256: _mm256_sub_ps(a[0].m256, b[0].m256) },
            f32x8 { m256: _mm256_sub_ps(a[1].m256, b[1].m256) },
            f32x8 { m256: _mm256_sub_ps(a[2].m256, b[2].m256) },
        ]
    }
}

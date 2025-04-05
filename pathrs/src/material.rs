use std::sync::Arc;

use bevy::math::{vec3, vec3a, Vec3, Vec3A};
use rand::Rng;

use crate::{renderer::Ray, scene::HitRecord};

pub trait Material {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vec3)>;

    fn emitted(&self, ray: &Ray, hit: &HitRecord) -> Vec3 {
        Vec3::ZERO
    }
}

pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Arc<Self> {
        Arc::new(Self { albedo })
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vec3)> {
        let mut scatter_dir = hit.normal + random_unit_vec();

        let epsilon = 1e-8;
        if scatter_dir.x.abs() < epsilon
            && scatter_dir.y.abs() < epsilon
            && scatter_dir.z.abs() < epsilon
        {
            scatter_dir = hit.normal;
        }

        let scattered = Ray {
            origin: hit.pos,
            direction: scatter_dir,
        };
        let attenuation = self.albedo;

        Some((scattered, attenuation))
    }
}

pub struct Metal {
    albedo: Vec3,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f32) -> Arc<Self> {
        Arc::new(Self { albedo, fuzz })
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vec3)> {
        let reflected = ray.direction.reflect(hit.normal);
        let reflected = reflected.normalize() + self.fuzz * random_unit_vec();
        let scattered = Ray {
            origin: hit.pos,
            direction: reflected,
        };
        let attenuation = self.albedo;

        Some((scattered, attenuation))
    }
}

pub struct Dielectric {
    refraction_index: f32,
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Arc<Self> {
        Arc::new(Self { refraction_index })
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vec3)> {
        let attenuation = vec3(1.0, 1.0, 1.0);

        let (ri, n) = if ray.direction.dot(hit.normal) < 0.0 {
            (1.0 / self.refraction_index, hit.normal)
        } else {
            (self.refraction_index, -hit.normal)
        };

        let unit_dir = ray.direction.normalize();

        let cos_theta = -unit_dir.dot(n).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let mut rng = rand::rng();
        let direction =
            if ri * sin_theta > 1.0 || reflectance(cos_theta, ri) > rng.random_range(0.0..=1.0) {
                unit_dir.reflect(n)
            } else {
                unit_dir.refract(n, ri)
            };

        let scattered = Ray {
            origin: hit.pos,
            direction,
        };

        Some((scattered, attenuation))
    }
}

fn reflectance(cos_theta: f32, ri: f32) -> f32 {
    let mut r0 = (1.0 - ri) / (1.0 + ri);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}

pub struct DiffuseLight {
    emitted: Vec3,
}

impl DiffuseLight {
    pub fn new(emitted: Vec3) -> Arc<Self> {
        Arc::new(Self { emitted })
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord) -> Option<(Ray, Vec3)> {
        None
    }

    fn emitted(&self, ray: &Ray, hit: &HitRecord) -> Vec3 {
        if ray.direction.dot(hit.normal) < 0.0 {
            self.emitted
        } else {
            Vec3::ZERO
        }
    }
}

#[inline(always)]
fn random_on_hemisphere(normal: Vec3A) -> Vec3A {
    let vec = random_on_unit_sphere();

    if vec.dot(normal) > 0.0 { vec } else { -vec }
}

#[inline(always)]
fn random_on_unit_sphere() -> Vec3A {
    loop {
        let vec = random_unit_vec();
        let len_squared = vec.length_squared();

        if 1e-30 < len_squared && len_squared <= 1.0 {
            return vec / len_squared.sqrt();
        }
    }
}

#[inline(always)]
fn random_unit_vec() -> Vec3A {
    let mut rng = rand::rng();

    vec3a(
        rng.random_range(-1.0..=1.0),
        rng.random_range(-1.0..=1.0),
        rng.random_range(-1.0..=1.0),
    )
}

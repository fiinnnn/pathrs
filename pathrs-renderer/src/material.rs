use glam::{vec3, vec3a, Vec3, Vec3A};
use rand::Rng;

use crate::{Ray, scene::HitRecord};

#[derive(Clone, Copy)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
}

impl Material {
    #[inline(always)]
    pub fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut fastrand::Rng) -> Option<(Ray, Vec3)> {
        match self {
            Material::Lambertian(l) => l.scatter(ray, hit, rng),
            Material::Metal(m) => m.scatter(ray, hit, rng),
            Material::Dielectric(d) => d.scatter(ray, hit, rng),
            Material::DiffuseLight(dl) => dl.scatter(ray, hit, rng),
        }
    }

    #[inline(always)]
    pub fn emitted(&self, ray: &Ray, hit: &HitRecord) -> Vec3 {
        match self {
            Material::Lambertian(l) => l.emitted(ray, hit),
            Material::Metal(m) => m.emitted(ray, hit),
            Material::Dielectric(d) => d.emitted(ray, hit),
            Material::DiffuseLight(dl) => dl.emitted(ray, hit),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }

    #[inline(always)]
    fn scatter(&self, _ray: &Ray, hit: &HitRecord, rng: &mut fastrand::Rng) -> Option<(Ray, Vec3)> {
        let mut scatter_dir = hit.normal + random_unit_vec(rng);

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

    #[inline(always)]
    fn emitted(&self, _ray: &Ray, _hit: &HitRecord) -> Vec3 {
        Vec3::ZERO
    }
}

impl From<Lambertian> for Material {
    fn from(val: Lambertian) -> Self {
        Material::Lambertian(val)
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Vec3,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f32) -> Self {
        Self { albedo, fuzz }
    }

    #[inline(always)]
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut fastrand::Rng) -> Option<(Ray, Vec3)> {
        let reflected = ray.direction.reflect(hit.normal);
        let reflected = reflected.normalize() + self.fuzz * random_unit_vec(rng);
        let scattered = Ray {
            origin: hit.pos,
            direction: reflected,
        };
        let attenuation = self.albedo;

        Some((scattered, attenuation))
    }

    #[inline(always)]
    fn emitted(&self, _ray: &Ray, _hit: &HitRecord) -> Vec3 {
        Vec3::ZERO
    }
}

impl From<Metal> for Material {
    fn from(val: Metal) -> Self {
        Material::Metal(val)
    }
}

#[derive(Clone, Copy)]
pub struct Dielectric {
    refraction_index: f32,
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Self {
        Self { refraction_index }
    }

    #[inline(always)]
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut fastrand::Rng) -> Option<(Ray, Vec3)> {
        let attenuation = vec3(1.0, 1.0, 1.0);

        let (ri, n) = if ray.direction.dot(hit.normal) < 0.0 {
            (1.0 / self.refraction_index, hit.normal)
        } else {
            (self.refraction_index, -hit.normal)
        };

        let unit_dir = ray.direction.normalize();

        let cos_theta = -unit_dir.dot(n).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let direction =
            if ri * sin_theta > 1.0 || reflectance(cos_theta, ri) > rng.f32() {
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

    #[inline(always)]
    fn emitted(&self, _ray: &Ray, _hit: &HitRecord) -> Vec3 {
        Vec3::ZERO
    }
}

impl From<Dielectric> for Material {
    fn from(val: Dielectric) -> Self {
        Material::Dielectric(val)
    }
}

#[inline(always)]
fn reflectance(cos_theta: f32, ri: f32) -> f32 {
    let mut r0 = (1.0 - ri) / (1.0 + ri);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}

#[derive(Clone, Copy)]
pub struct DiffuseLight {
    emitted: Vec3,
}

impl DiffuseLight {
    pub fn new(emitted: Vec3) -> Self {
        Self { emitted }
    }

    #[inline(always)]
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord, _rng: &mut fastrand::Rng) -> Option<(Ray, Vec3)> {
        None
    }

    #[inline(always)]
    fn emitted(&self, ray: &Ray, hit: &HitRecord) -> Vec3 {
        if ray.direction.dot(hit.normal) < 0.0 {
            self.emitted
        } else {
            Vec3::ZERO
        }
    }
}

impl From<DiffuseLight> for Material {
    fn from(val: DiffuseLight) -> Self {
        Material::DiffuseLight(val)
    }
}

#[inline(always)]
fn random_on_hemisphere(normal: Vec3, rng: &mut fastrand::Rng) -> Vec3 {
    let vec = random_on_unit_sphere(rng);

    if vec.dot(normal) > 0.0 { vec } else { -vec }
}

#[inline(always)]
fn random_on_unit_sphere(rng: &mut fastrand::Rng) -> Vec3 {
    loop {
        let vec = random_unit_vec(rng);
        let len_squared = vec.length_squared();

        if 1e-30 < len_squared && len_squared <= 1.0 {
            return vec / len_squared.sqrt();
        }
    }
}

#[inline(always)]
fn random_unit_vec(rng: &mut fastrand::Rng) -> Vec3 {
    use fastrand_contrib::RngExt;

    vec3(
        rng.f32_range(-1.0..=1.0),
        rng.f32_range(-1.0..=1.0),
        rng.f32_range(-1.0..=1.0),
    ).normalize()
}

use glam::vec3;

use crate::{
    HitRecord, Ray,
    geometry::{Sphere, Spheres, SpheresSIMD, Triangle, Triangles, TrianglesSIMD},
    material::{Dielectric, DiffuseLight, Lambertian, Material, Metal},
};

#[cfg(feature = "simd")]
use crate::simd::*;

#[derive(Default, Clone)]
pub struct Scene {
    pub materials: Vec<Material>,

    triangles: Triangles,
    spheres: Spheres,

    #[cfg(feature = "simd")]
    spheres_simd: SpheresSIMD,
    #[cfg(feature = "simd")]
    triangles_simd: TrianglesSIMD,
}

impl Scene {
    pub fn add_triangle(&mut self, tri: Triangle) {
        self.triangles.push(tri);
    }

    pub fn add_triangles(&mut self, tris: &[Triangle]) {
        for tri in tris {
            self.triangles.push(tri.clone());
        }
    }

    pub fn add_sphere(&mut self, sphere: Sphere) {
        self.spheres.push(sphere);
    }

    pub fn add_material<M: Into<Material>>(&mut self, mat: M) -> u32 {
        self.materials.push(mat.into());
        (self.materials.len() - 1) as u32
    }

    #[cfg(feature = "simd")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn collect_simd(&mut self) {
        self.triangles_simd = TrianglesSIMD::from_tris(self.triangles.clone());
        self.spheres_simd = SpheresSIMD::from_spheres(self.spheres.clone());
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn closest_hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut res = None;
        let mut tmax = tmax;

        #[cfg(feature = "simd")]
        let r_o = Vec3x8::from(ray.origin);
        #[cfg(feature = "simd")]
        let r_d = Vec3x8::from(ray.direction);

        #[cfg(not(feature = "simd"))]
        if let Some(hit) = self.spheres.intersect_spheres(ray, tmin, tmax) {
            tmax = hit.t;
            res = Some(hit);
        }

        #[cfg(feature = "simd")]
        if let Some(hit) = self.spheres_simd.intersect(ray, tmin, tmax, r_o, r_d) {
            tmax = hit.t;
            res = Some(hit);
        }

        #[cfg(not(feature = "simd"))]
        if let Some(hit) = self.triangles.intersect(ray, tmin, tmax) {
            res = Some(hit);
        }

        #[cfg(feature = "simd")]
        if let Some(hit) = self.triangles_simd.intersect(ray, tmin, tmax, r_o, r_d) {
            res = Some(hit);
        }

        res
    }
}

pub fn test_scene(scene: &mut Scene) {
    let red = scene.add_material(Metal::new(vec3(0.85, 0.30, 0.30), 0.2));
    let white = scene.add_material(Lambertian::new(vec3(0.73, 0.73, 0.73)));
    let green = scene.add_material(Lambertian::new(vec3(0.12, 0.45, 0.15)));
    let light = scene.add_material(DiffuseLight::new(vec3(5.0, 5.0, 5.0)));

    scene.add_triangles(&Triangle::quad(
        vec3(-20.0, 0.0, 0.0),
        vec3(-20.0, 0.0, -40.0),
        vec3(-20.0, 40.0, 0.0),
        vec3(-20.0, 40.0, -400.0),
        green,
    ));

    scene.add_triangles(&Triangle::quad(
        vec3(20.0, 0.0, 0.0),
        vec3(20.0, 0.0, -40.0),
        vec3(20.0, 40.0, 0.0),
        vec3(20.0, 40.0, -40.0),
        red,
    ));

    scene.add_triangles(&Triangle::quad(
        vec3(-20.0, 0.0, 0.0),
        vec3(20.0, 0.0, 0.0),
        vec3(-20.0, 0.0, -40.0),
        vec3(20.0, 0.0, -40.0),
        white,
    ));

    scene.add_triangles(&Triangle::quad(
        vec3(-20.0, 40.0, 0.0),
        vec3(20.0, 40.0, 0.0),
        vec3(-20.0, 40.0, -40.0),
        vec3(20.0, 40.0, -40.0),
        white,
    ));

    scene.add_triangles(&Triangle::quad(
        vec3(-20.0, 0.0, -40.0),
        vec3(20.0, 0.0, -40.0),
        vec3(-20.0, 40.0, -40.0),
        vec3(20.0, 40.0, -40.0),
        white,
    ));

    scene.add_triangles(&Triangle::quad(
        vec3(-20.0, 0.0, 0.0),
        vec3(20.0, 0.0, 0.0),
        vec3(-20.0, 40.0, 0.0),
        vec3(20.0, 40.0, 0.0),
        white,
    ));

    scene.add_triangles(&Triangle::quad(
        vec3(-5.0, 39.99, -15.0),
        vec3(5.0, 39.99, -15.0),
        vec3(-5.0, 39.99, -25.0),
        vec3(5.0, 39.99, -25.0),
        light,
    ));

    let sphere = scene.add_material(Dielectric::new(1.50));
    scene.add_sphere(Sphere::new(vec3(-6.0, 8.0, -26.0), 5.0, sphere));

    // let sphere_inner = scene.add_material(Dielectric::new(1.00 / 1.50));
    // scene.add_object(Sphere::new(vec3(-7.0, 9.0, -26.0), 4.0, sphere_inner));

    let mirror = scene.add_material(Metal::new(vec3(0.82, 0.82, 0.82), 0.01));
    scene.add_triangles(&Triangle::quad(
        vec3(7.5, 0.0, -35.0),
        vec3(12.5, 0.0, -31.0),
        vec3(7.5, 20.0, -35.0),
        vec3(12.5, 20.0, -31.0),
        mirror,
    ));

    let metal = scene.add_material(Metal::new(vec3(0.72, 0.45, 0.12), 0.64));
    scene.add_sphere(Sphere::new(vec3(-4.0, 20.0, -24.0), 2.5, metal));

    let glass = scene.add_material(Dielectric::new(1.5));
    scene.add_sphere(Sphere::new(vec3(-17.0, 3.0, -37.0), 1.5, glass));

    let light_sphere = scene.add_material(DiffuseLight::new(vec3(3.5, 1.8, 0.2)));
    scene.add_sphere(Sphere::new(vec3(-17.0, 3.0, -37.0), 1.0, light_sphere));

    #[cfg(feature = "simd")]
    scene.collect_simd();
}

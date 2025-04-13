use fastrand::Rng;
use glam::{vec3, Vec3, Vec4};

use crate::{camera::Camera, metrics::RenderPassMetrics, scene::Scene, Ray};

use super::Renderer;

pub struct CPURenderer;

impl Renderer for CPURenderer {
    fn new() -> CPURenderer {
        CPURenderer {}
    }

    fn render_pass(
        &mut self,
        camera: &Camera,
        scene: &Scene,
        acc: &mut [Vec4],
        rng: &mut Rng,
    ) -> RenderPassMetrics {
        let size = camera.screen_size;
        let width = size.x as usize;
        let height = size.y as usize;

        let mut ray_count = 0;

        for y in 0..height {
            for x in 0..width {
                acc[x + y * width] += per_pixel(x, y, camera, scene, &mut ray_count, rng);
            }
        }

        RenderPassMetrics { ray_count }
    }
}

const MAX_DEPTH: usize = 10;

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
fn per_pixel(
    x: usize,
    y: usize,
    camera: &Camera,
    scene: &Scene,
    ray_count: &mut usize,
    rng: &mut fastrand::Rng,
) -> Vec4 {
    let mut res = Vec4::ZERO;
    let ray = camera.get_ray(x, y);
    let col = trace_ray(&ray, scene, 0, ray_count, rng);
    res += col.extend(1.0);
    res
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
fn trace_ray(
    ray: &Ray,
    scene: &Scene,
    depth: usize,
    ray_count: &mut usize,
    rng: &mut fastrand::Rng,
) -> Vec3 {
    if depth == MAX_DEPTH {
        return Vec3::ZERO;
    }

    *ray_count += 1;

    if let Some(hit) = scene.closest_hit(ray, 0.0001, f32::MAX) {
        let mat = scene.materials[hit.material as usize];
        let emitted = mat.emitted(ray, &hit);

        let scattered = if let Some((scattered, attenuation)) = mat.scatter(ray, &hit, rng) {
            attenuation * trace_ray(&scattered, scene, depth + 1, ray_count, rng)
        } else {
            Vec3::ZERO
        };

        return emitted + scattered;
    }

    let dir = ray.direction.normalize();
    let a = 0.5 * (dir.y + 1.0);
    (1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0)
}

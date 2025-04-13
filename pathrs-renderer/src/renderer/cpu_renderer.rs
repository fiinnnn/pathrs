use fastrand::Rng;
use glam::{Vec3, Vec4, vec3};

use crate::{Ray, camera::Camera, metrics::RenderPassMetrics, scene::Scene};

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
        let mut metrics = RenderPassMetrics::default();

        let size = camera.screen_size;
        let width = size.x as usize;
        let height = size.y as usize;

        for y in 0..height {
            for x in 0..width {
                acc[x + y * width] += per_pixel(x, y, camera, scene, rng, &mut metrics);
            }
        }

        metrics
    }
}

const MAX_DEPTH: usize = 10;

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
fn per_pixel(
    x: usize,
    y: usize,
    camera: &Camera,
    scene: &Scene,
    rng: &mut fastrand::Rng,
    metrics: &mut RenderPassMetrics,
) -> Vec4 {
    let ray = camera.get_ray(x, y);
    trace_ray(&ray, scene, 0, rng, metrics).extend(1.0)
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
fn trace_ray(
    ray: &Ray,
    scene: &Scene,
    depth: usize,
    rng: &mut fastrand::Rng,
    metrics: &mut RenderPassMetrics,
) -> Vec3 {
    metrics.ray_count += 1;

    if depth == MAX_DEPTH {
        metrics.add_depth(depth);
        return Vec3::ZERO;
    }

    if let Some(hit) = scene.closest_hit(ray, 0.0001, f32::MAX) {
        let mat = scene.materials[hit.material as usize];
        let emitted = mat.emitted(ray, &hit);

        let scattered = if let Some((scattered, attenuation)) = mat.scatter(ray, &hit, rng) {
            attenuation * trace_ray(&scattered, scene, depth + 1, rng, metrics)
        } else {
            metrics.add_depth(depth);
            Vec3::ZERO
        };

        return emitted + scattered;
    } else {
        metrics.add_depth(depth);
    }

    let dir = ray.direction.normalize();
    let a = 0.5 * (dir.y + 1.0);
    (1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0)
}

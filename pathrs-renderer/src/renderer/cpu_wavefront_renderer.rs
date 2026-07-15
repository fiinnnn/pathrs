use fastrand::Rng;
use glam::{Vec3, Vec4};

use crate::{HitRecord, Ray, camera::Camera, metrics::RenderPassMetrics, scene::Scene};

use super::Renderer;

pub struct CPUWavefrontRenderer {
    ray_buf: Vec<(bool, Ray, usize)>,
    scatter_buf: Vec<(bool, Ray, usize)>,
    hit_buf: Vec<Option<HitRecord>>,
    active_rays: usize,
}

impl Renderer for CPUWavefrontRenderer {
    fn new() -> Self {
        CPUWavefrontRenderer {
            ray_buf: Vec::new(),
            scatter_buf: Vec::new(),
            hit_buf: Vec::new(),
            active_rays: 0,
        }
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

        if self.ray_buf.len() != width * height {
            self.ray_buf
                .resize(width * height, (false, Ray::new(Vec3::ZERO, Vec3::ZERO), 0));
            self.scatter_buf
                .resize(width * height, (false, Ray::new(Vec3::ZERO, Vec3::ZERO), 0));
            self.hit_buf.resize(width * height, None);
        }

        let mut throughput = vec![Vec3::ONE; width * height];
        let mut color = vec![Vec3::ZERO; width * height];

        self.active_rays = 0;
        generate_primary_rays(
            width,
            height,
            camera,
            &mut self.ray_buf,
            &mut self.active_rays,
        );

        while self.active_rays > 0 {
            extend_rays(scene, &self.ray_buf, &mut self.hit_buf, &mut self.active_rays);

            metrics.ray_count += self.active_rays;

            shade(
                scene,
                &mut self.ray_buf,
                &self.hit_buf,
                &mut self.scatter_buf,
                &mut throughput,
                &mut color,
                rng
            );

            std::mem::swap(&mut self.scatter_buf, &mut self.ray_buf);
        }

        for (i, v) in color.iter().enumerate() {
            acc[i] += v.extend(1.0);
        }

        metrics
    }
}

fn generate_primary_rays(
    width: usize,
    height: usize,
    camera: &Camera,
    ray_buf: &mut [(bool, Ray, usize)],
    active_rays: &mut usize,
) {
    for y in 0..height {
        for x in 0..width {
            ray_buf[x + y * width] = (true, camera.get_ray(x, y), 0);
            *active_rays += 1;
        }
    }
}

fn extend_rays(scene: &Scene, ray_buf: &[(bool, Ray, usize)], hit_buf: &mut [Option<HitRecord>], active_rays: &mut usize) {
    let mut r_cnt = 0;
    for (i, (active, ray, _)) in ray_buf.iter().enumerate() {
        if !active {
            continue;
        }

        r_cnt += 1;
        hit_buf[i] = scene.closest_hit(ray, 0.0001, f32::MAX);
    }

    *active_rays = r_cnt;
}

fn shade(
    scene: &Scene,
    ray_buf: &mut [(bool, Ray, usize)],
    hit_buf: &[Option<HitRecord>],
    scatter_buf: &mut [(bool, Ray, usize)],
    throughput: &mut [Vec3],
    color: &mut [Vec3],
    rng: &mut Rng,
) {
    for (i, hit) in hit_buf.iter().enumerate() {
        let (active, ref ray, depth) = ray_buf[i];

        if !active {
            continue;
        }

        let Some(hit) = hit else {
            ray_buf[i].0 = false;
            scatter_buf[i].0 = false;
            continue;
        };

        let mat = scene.materials[hit.material as usize];

        let emitted = mat.emitted(ray, hit);
        color[i] += throughput[i] * emitted;

        if depth <= 10 {
            if let Some((scattered, attenuation)) = mat.scatter(ray, hit, rng) {
                throughput[i] *= attenuation;
                scatter_buf[i] = (true, scattered, depth + 1);
                continue;
            }
        }

        ray_buf[i].0 = false;
        scatter_buf[i].0 = false;
    }
}

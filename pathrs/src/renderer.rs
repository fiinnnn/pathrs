use std::time::{Duration, Instant};

use async_channel::{Receiver, Sender};
use bevy::math::{vec3, UVec2, Vec3, Vec3A, Vec4};

use crate::{
    camera::Camera,
    material::{Dielectric, DiffuseLight, Lambertian, Metal},
    scene::{Scene, Sphere, Triangle},
};

#[derive(Clone, Copy)]
pub enum RendererCmd {
    Resize { size: UVec2 },
}

#[derive(Clone, Default)]
pub struct RenderResult {
    pub image_data: Vec<Vec4>,
    pub image_size: UVec2,
    pub render_time: Duration,
    pub rays_per_second: f64,
}

pub struct Renderer {
    camera: Camera,
    scene: Scene,
    size: UVec2,

    cmd_rx: Receiver<RendererCmd>,
    input: triple_buffer::Input<RenderResult>,

    samples: usize,
}

impl Renderer {
    pub fn new(
        size: UVec2,
    ) -> (
        Renderer,
        Sender<RendererCmd>,
        triple_buffer::Output<RenderResult>,
    ) {
        let camera =
            crate::camera::Camera::new(vec3(0.0, 15.0, -2.5), vec3(0.0, 15.0, -5.5), 90.0, size);

        let mut scene = crate::scene::Scene::default();

        let red = Metal::new(vec3(0.85, 0.30, 0.30), 0.2);
        let white = Lambertian::new(vec3(0.73, 0.73, 0.73));
        let green = Lambertian::new(vec3(0.12, 0.45, 0.15));
        let light = DiffuseLight::new(vec3(5.0, 5.0, 5.0));

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
            white.clone(),
        ));

        scene.add_triangles(&Triangle::quad(
            vec3(-20.0, 40.0, 0.0),
            vec3(20.0, 40.0, 0.0),
            vec3(-20.0, 40.0, -40.0),
            vec3(20.0, 40.0, -40.0),
            white.clone(),
        ));

        scene.add_triangles(&Triangle::quad(
            vec3(-20.0, 0.0, -40.0),
            vec3(20.0, 0.0, -40.0),
            vec3(-20.0, 40.0, -40.0),
            vec3(20.0, 40.0, -40.0),
            white.clone(),
        ));

        scene.add_triangles(&Triangle::quad(
            vec3(-20.0, 0.0, 0.0),
            vec3(20.0, 0.0, 0.0),
            vec3(-20.0, 40.0, 0.0),
            vec3(20.0, 40.0, 0.0),
            white.clone(),
        ));

        scene.add_triangles(&Triangle::quad(
            vec3(-5.0, 39.99, -15.0),
            vec3(5.0, 39.99, -15.0),
            vec3(-5.0, 39.99, -25.0),
            vec3(5.0, 39.99, -25.0),
            light,
        ));

        let sphere = Dielectric::new(1.50);
        let sphere_inner = Dielectric::new(1.00 / 1.50);
        scene.add_object(Sphere::new(vec3(-6.0, 8.0, -26.0), 5.0, sphere));
        // scene.add_object(Sphere::new(vec3(-7.0, 9.0, -26.0), 4.0, sphere_inner));

        let mirror = Metal::new(vec3(0.82, 0.82, 0.82), 0.01);
        scene.add_triangles(&Triangle::quad(
            vec3(7.5, 0.0, -35.0),
            vec3(12.5, 0.0, -31.0),
            vec3(7.5, 20.0, -35.0),
            vec3(12.5, 20.0, -31.0),
            mirror,
        ));

        let metal = Metal::new(vec3(0.72, 0.45, 0.12), 0.64);
        scene.add_object(Sphere::new(vec3(-4.0, 20.0, -24.0), 2.5, metal));

        scene.add_object(Sphere::new(
            vec3(-17.0, 3.0, -37.0),
            1.5,
            Dielectric::new(1.5),
        ));
        scene.add_object(Sphere::new(
            vec3(-17.0, 3.0, -37.0),
            1.0,
            DiffuseLight::new(vec3(3.5, 1.8, 0.2)),
        ));

        let (input, output) = triple_buffer::triple_buffer(&RenderResult::default());

        let (cmd_tx, cmd_rx) = async_channel::bounded(1);

        (
            Renderer {
                camera,
                scene,
                size,
                cmd_rx,
                input,
                samples: 0,
            },
            cmd_tx,
            output,
        )
    }

    pub async fn run_renderer(mut self) {
        let mut acc = vec![Vec4::ZERO];

        loop {
            match self.cmd_rx.try_recv() {
                Ok(RendererCmd::Resize { size }) => {
                    self.size = size;
                    self.camera.resize(size);
                    self.samples = 0;
                }
                Err(_) => (),
            };

            let RenderResult {
                image_data,
                image_size,
                render_time,
                rays_per_second,
            } = self.input.input_buffer_mut();

            let start = Instant::now();

            *image_size = self.size;
            self.samples += 1;

            let (width, height) = (image_size.x as usize, image_size.y as usize);

            let len = width * height;
            if image_data.len() != len {
                image_data.resize(len, Vec4::ZERO);
                acc.clear();
                acc.resize(len, Vec4::ZERO);
            }

            let mut ray_count = 0;

            for y in 0..height {
                for x in 0..width {
                    acc[x + y * width] +=
                        per_pixel(x, y, &self.camera, &self.scene, &mut ray_count);
                    image_data[x + y * width] = acc[x + y * width] / self.samples as f32;
                }
            }

            let end = start.elapsed();
            *render_time = end;
            *rays_per_second = ray_count as f64 / end.as_secs_f64();

            self.input.publish();
        }
    }
}

const MAX_DEPTH: usize = 10;

fn per_pixel(x: usize, y: usize, camera: &Camera, scene: &Scene, ray_count: &mut usize) -> Vec4 {
    let mut res = Vec4::ZERO;
    let ray = camera.get_ray(x, y);
    let col = trace_ray(&ray, scene, 0, ray_count);
    res += col.extend(1.0);
    res
}

fn trace_ray(ray: &Ray, scene: &Scene, depth: usize, ray_count: &mut usize) -> Vec3 {
    if depth == MAX_DEPTH {
        return Vec3::ZERO;
    }

    *ray_count += 1;

    if let Some(hit) = scene.closest_hit(ray, 0.0001, f32::MAX) {
        let emitted = hit.material.emitted(ray, &hit);

        let scattered = if let Some((scattered, attenuation)) = hit.material.scatter(ray, &hit) {
            attenuation * trace_ray(&scattered, scene, depth + 1, ray_count)
        } else {
            Vec3::ZERO
        };

        return emitted + scattered;
    }

    let dir = ray.direction.normalize();
    let a = 0.5 * (dir.y + 1.0);
    (1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0)
}

pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}

impl Ray {
    pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
        Self { origin, direction }
    }
}

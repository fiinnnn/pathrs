use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use camera::Camera;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use fastrand::Rng;
use glam::{UVec2, Vec3, Vec4, uvec2, vec3};
use material::{Dielectric, DiffuseLight, Lambertian, Metal};
use scene::{Scene, Sphere, Triangle};

mod camera;
mod material;
mod scene;

#[cfg(feature = "simd")]
mod simd;

static SCREEN_RESIZE: Mutex<Option<UVec2>> = Mutex::new(None);

pub fn resize_render_target(width: u32, height: u32) {
    if let Ok(mut l) = SCREEN_RESIZE.lock() {
        l.replace(uvec2(width, height));
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RendererCmd {
    Stop,
}

#[derive(Clone, Default)]
pub struct RenderResult {
    pub image_data: Vec<[f32; 4]>,
    pub image_size: UVec2,
    pub render_time: Duration,
    pub rays_per_second: f64,
}

pub struct Renderer {
    camera: Camera,
    scene: Scene,
    size: UVec2,

    cmd_rx: Receiver<RendererCmd>,
    out_buffer: triple_buffer::Input<RenderResult>,

    samples: usize,
}

impl Renderer {
    pub fn new(
        width: u32,
        height: u32,
    ) -> (
        Renderer,
        Sender<RendererCmd>,
        triple_buffer::Output<RenderResult>,
    ) {
        let size = uvec2(width, height);

        let camera = Camera::new(vec3(0.0, 15.0, -2.5), vec3(0.0, 15.0, -5.5), 90.0, size);

        let mut scene = Scene::default();

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
        scene.add_object(Sphere::new(vec3(-6.0, 8.0, -26.0), 5.0, sphere));

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
        scene.add_object(Sphere::new(vec3(-4.0, 20.0, -24.0), 2.5, metal));

        let glass = scene.add_material(Dielectric::new(1.5));
        scene.add_object(Sphere::new(vec3(-17.0, 3.0, -37.0), 1.5, glass));

        let light_sphere = scene.add_material(DiffuseLight::new(vec3(3.5, 1.8, 0.2)));
        scene.add_object(Sphere::new(vec3(-17.0, 3.0, -37.0), 1.0, light_sphere));

        #[cfg(feature = "simd")]
        scene.collect_simd();

        let (input, output) = triple_buffer::triple_buffer(&RenderResult::default());

        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();

        (
            Renderer {
                camera,
                scene,
                size,
                cmd_rx,
                out_buffer: input,
                samples: 0,
            },
            cmd_tx,
            output,
        )
    }

    pub fn start_thread(self) {
        thread::spawn(|| self.run_loop());
    }

    fn run_loop(mut self) {
        println!("TID: {}", unsafe { libc::syscall(libc::SYS_gettid) });

        let mut rng = fastrand::Rng::new();
        let mut acc = vec![Vec4::ZERO];

        loop {
            if let Some(size) = SCREEN_RESIZE.lock().ok().and_then(|mut l| l.take()) {
                self.size = size;
                self.camera.resize(size);
                self.samples = 0;
            }

            match self.cmd_rx.try_recv() {
                Ok(RendererCmd::Stop) | Err(TryRecvError::Disconnected) => return,
                _ => (),
            }

            self.render_pass(&mut acc, &mut rng);
        }
    }

    pub fn render_image(mut self, samples_per_pixel: u32) -> Vec<[f32; 4]> {
        let mut rng = fastrand::Rng::new();
        let mut acc = vec![Vec4::ZERO];

        for i in 0..samples_per_pixel {
            if i % 10 == 0 {
                println!("{i}/{samples_per_pixel}");
            }

            self.render_pass(&mut acc, &mut rng);
        }

        acc.into_iter()
            .map(|p| (p / samples_per_pixel as f32).to_array())
            .collect()
    }

    fn render_pass(&mut self, acc: &mut Vec<Vec4>, rng: &mut Rng) {
        let RenderResult {
            image_data,
            image_size,
            render_time,
            rays_per_second,
        } = self.out_buffer.input_buffer_mut();

        let start = Instant::now();

        *image_size = self.size;
        self.samples += 1;

        let (width, height) = (image_size.x as usize, image_size.y as usize);

        let len = width * height;
        if image_data.len() != len {
            image_data.resize(len, [0.0; 4]);
            acc.clear();
            acc.resize(len, Vec4::ZERO);
        }

        let mut ray_count = 0;

        for y in 0..height {
            for x in 0..width {
                acc[x + y * width] +=
                    per_pixel(x, y, &self.camera, &self.scene, &mut ray_count, rng);
                image_data[x + y * width] = (acc[x + y * width] / self.samples as f32).to_array();
            }
        }

        let end = start.elapsed();
        *render_time = end;
        *rays_per_second = ray_count as f64 / end.as_secs_f64();

        self.out_buffer.publish();
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

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }
}

use std::{thread, time::Instant};

use camera::Camera;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use glam::{UVec2, Vec3, Vec4, uvec2, vec3};
use metrics::RenderPassMetrics;
use scene::Scene;

use crate::renderer::Renderer;

mod camera;
mod geometry;
mod material;
pub mod renderer;
mod scene;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "simd")]
mod simd;

#[derive(Clone, Copy, PartialEq)]
pub enum RendererCmd {
    Stop,
    Resize { width: u32, height: u32 },
}

#[derive(Clone, Default)]
pub struct RenderResult {
    pub image_data: Vec<[f32; 4]>,
    pub image_size: UVec2,
    pub render_pass_metrics: RenderPassMetrics,
}

pub struct RenderSystem<R: Renderer> {
    camera: Camera,
    scene: Scene,
    size: UVec2,

    renderer: R,
    samples: usize,

    cmd_rx: Receiver<RendererCmd>,
    input: triple_buffer::Input<RenderResult>,
}

impl<R: Renderer> RenderSystem<R> {
    pub fn new(
        width: u32,
        height: u32,
    ) -> (
        Self,
        Sender<RendererCmd>,
        triple_buffer::Output<RenderResult>,
    ) {
        let size = uvec2(width, height);

        let camera = Camera::new(vec3(0.0, 15.0, -2.5), vec3(0.0, 15.0, -5.5), 90.0, size);

        let mut scene = Scene::default();
        scene::test_scene(&mut scene);

        let (input, output) = triple_buffer::triple_buffer(&RenderResult::default());

        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();

        (
            Self {
                camera,
                scene,
                size,

                renderer: R::new(),
                samples: 0,

                cmd_rx,
                input,
            },
            cmd_tx,
            output,
        )
    }

    pub fn start_thread(self) {
        thread::spawn(|| self.run_render_loop());
    }

    fn receive_commands(&mut self) -> bool {
        let mut last_resize = None;
        loop {
            match self.cmd_rx.try_recv() {
                Ok(RendererCmd::Resize { width, height }) => {
                    last_resize = Some(uvec2(width, height));
                }
                Ok(RendererCmd::Stop) => return false,
                Err(TryRecvError::Disconnected) => return false,
                Err(TryRecvError::Empty) => break,
            }
        }

        if let Some(size) = last_resize {
            self.size = size;
            self.camera.resize(size);
            self.samples = 0;
        }

        true
    }

    fn run_render_loop(mut self) {
        println!("TID: {}", unsafe { libc::syscall(libc::SYS_gettid) });

        let mut rng = fastrand::Rng::new();
        let mut acc = vec![Vec4::ZERO; (self.size.x * self.size.y) as usize];

        loop {
            if !self.receive_commands() {
                return;
            }

            let RenderResult {
                image_data,
                image_size,
                render_pass_metrics,
            } = self.input.input_buffer_mut();

            let len = (self.size.x * self.size.y) as usize;
            if image_data.len() != len {
                image_data.resize(len, [0.0; 4]);
                acc.clear();
                acc.resize(len, Vec4::ZERO);
            }

            *image_size = self.size;

            self.samples += 1;

            let start = Instant::now();

            let mut metrics =
                self.renderer
                    .render_pass(&self.camera, &self.scene, &mut acc, &mut rng);

            metrics.render_time = start.elapsed();
            *render_pass_metrics = metrics;

            for (i, acc_sample) in acc.iter().enumerate() {
                image_data[i] = (acc_sample / self.samples as f32).to_array();
            }

            self.input.publish();
        }
    }

    pub fn render_image(mut self, samples_per_pixel: u32) -> Vec<[f32; 4]> {
        let mut rng = fastrand::Rng::new();
        let mut acc = vec![Vec4::ZERO; (self.size.x * self.size.y) as usize];

        for i in 0..samples_per_pixel {
            if i % 10 == 0 {
                println!("{i}/{samples_per_pixel}");
            }

            let _metrics = self
                .renderer
                .render_pass(&self.camera, &self.scene, &mut acc, &mut rng);
        }

        acc.into_iter()
            .map(|p| (p / samples_per_pixel as f32).to_array())
            .collect()
    }
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

#[derive(Clone)]
pub struct HitRecord {
    pub pos: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material: u32,
}

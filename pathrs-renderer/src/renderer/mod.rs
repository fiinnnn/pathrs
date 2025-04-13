use fastrand::Rng;
use glam::Vec4;

use crate::{camera::Camera, metrics::RenderPassMetrics, scene::Scene};

mod cpu_renderer;

pub use cpu_renderer::CPURenderer;

pub trait Renderer: Sync + Send + 'static {
    fn new() -> Self;

    fn render_pass(
        &mut self,
        camera: &Camera,
        scene: &Scene,
        acc: &mut [Vec4],
        rng: &mut Rng,
    ) -> RenderPassMetrics;
}

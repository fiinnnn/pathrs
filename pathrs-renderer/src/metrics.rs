use std::time::Duration;

use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct RendererMetrics {
    pub capacity: usize,
    pub passes: AllocRingBuffer<RenderPassMetrics>,
}

impl RendererMetrics {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            passes: AllocRingBuffer::new(capacity),
        }
    }

    pub fn add_pass(&mut self, pass: RenderPassMetrics) {
        self.passes.push(pass);
    }

    pub fn iter(&self) -> impl Iterator<Item = &RenderPassMetrics> {
        self.passes.iter()
    }

    pub fn render_times(&self) -> impl Iterator<Item = f32> {
        self.passes.iter().map(|m| m.render_time.as_millis() as f32)
    }

    pub fn rays_per_second(&self) -> impl Iterator<Item = f32> {
        self.passes
            .iter()
            .map(|p| p.ray_count as f32 / p.render_time.as_secs_f32())
    }

    pub fn average_depth_histogram(&self) -> [f32; 11] {
        let mut histogram = [0.0f32; 11];
        let mut total_rays = 0.0;

        for pass in &self.passes {
            for (i, &v) in pass.ray_depth_histogram.iter().enumerate() {
                histogram[i] += v as f32;
            }
            total_rays += pass.ray_depth_histogram_count as f32;
        }

        if total_rays > 0.0 {
            for v in &mut histogram {
                *v /= total_rays;
            }
        }

        histogram
    }
}

#[derive(Default, Clone, Copy)]
pub struct RenderPassMetrics {
    pub ray_count: usize,
    pub ray_depth_histogram: [usize; 11],
    pub ray_depth_histogram_count: usize,
    pub render_time: Duration,
}

impl RenderPassMetrics {
    #[inline(always)]
    pub fn add_depth(&mut self, depth: usize) {
        self.ray_depth_histogram[depth] += 1;
        self.ray_depth_histogram_count += 1;
    }

    pub fn combine(&mut self, other: &Self) {
        self.ray_count += other.ray_count;

        for i in 0..self.ray_depth_histogram.len() {
            self.ray_depth_histogram[i] += other.ray_depth_histogram[i];
        }
        self.ray_depth_histogram_count += other.ray_depth_histogram_count;
    }

    pub fn combined<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut total = Self::default();
        for item in iter {
            total.combine(&item);
        }
        total
    }
}

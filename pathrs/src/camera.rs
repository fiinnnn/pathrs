use std::f32::consts::PI;

use bevy::math::{UVec2, Vec3, Vec3A, vec3};

use crate::renderer::Ray;

#[derive(Clone)]
pub struct Camera {
    screen_upper_left: Vec3A,
    screen_right: Vec3A,
    screen_down: Vec3A,

    look_from: Vec3A,
    look_at: Vec3,
    v_up: Vec3,

    u: Vec3,
    v: Vec3,
    w: Vec3,
    vfov: f32,
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, vfov: f32, size: UVec2) -> Camera {
        let mut camera = Camera {
            screen_upper_left: Vec3A::ZERO,
            screen_right: Vec3A::ZERO,
            screen_down: Vec3A::ZERO,

            look_from: look_from.into(),
            look_at,
            v_up: vec3(0.0, 1.0, 0.0),

            u: Vec3::ZERO,
            v: Vec3::ZERO,
            w: Vec3::ZERO,
            vfov,
        };

        camera.resize(size);

        camera
    }

    pub fn resize(&mut self, size: UVec2) {
        let aspect_ratio = size.x as f32 / size.y as f32;

        let focal_length = (Vec3::from(self.look_from) - self.look_at).length();
        let theta = (self.vfov * PI) / 180.0;
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * aspect_ratio;

        self.w = (Vec3::from(self.look_from) - self.look_at).normalize();
        self.u = self.v_up.cross(self.w).normalize();
        self.v = self.w.cross(self.u);

        let viewport_u = viewport_width * self.u;
        let viewport_v = viewport_height * -self.v;

        self.screen_right = (viewport_u / size.x as f32).into();
        self.screen_down = (viewport_v / size.y as f32).into();

        self.screen_upper_left = self.look_from
            - (focal_length * Vec3A::from(self.w))
            - (Vec3A::from(viewport_u) / 2.0)
            - (Vec3A::from(viewport_v) / 2.0)
            + 0.5 * (self.screen_right + self.screen_down);
    }

    pub fn get_ray(&self, x: usize, y: usize) -> Ray {
        let jitter_x = rand::random_range(-0.5..=0.5);
        let jitter_y = rand::random_range(-0.5..=0.5);

        let pixel_pos = self.screen_upper_left
            + ((x as f32 + jitter_x) * self.screen_right)
            + ((y as f32 + jitter_y) * self.screen_down);

        Ray {
            origin: self.look_from,
            direction: pixel_pos - self.look_from,
        }
    }
}

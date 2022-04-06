use glam::Vec3;

pub struct Camera {
    pub origin: Vec3,
    pub forward: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub width: f32,
    pub height: f32,
}

impl Camera {
    pub fn as_viewport(&self) -> pathrs_shared::Viewport {
        let theta = self.fov.to_radians();
        let d = (theta / 2.0).tan();
        let vp_height = 2.0 * d;
        let vp_width = self.aspect_ratio * vp_height;

        let right = self.forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let up = self.forward.cross(right).normalize();

        let horizontal = vp_width * right;
        let vertical = vp_height * up;
        let lower_left = self.origin - horizontal / 2.0 - vertical / 2.0 + self.forward;

        pathrs_shared::Viewport::new(
            self.origin,
            horizontal,
            vertical,
            lower_left,
            self.width,
            self.height,
        )
    }
}

pub struct CameraController {}

impl CameraController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(
        &self,
        camera: &mut Camera,
        input: &winit_input_helper::WinitInputHelper,
        window: &winit::window::Window,
    ) {
    }
}

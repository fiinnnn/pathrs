use glam::{Mat4, Vec3, Vec3Swizzles};
use imgui::*;
use winit::event::{KeyboardInput, VirtualKeyCode, WindowEvent};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    pos: [f32; 4],
    p0: [f32; 4],
    p0p1: [f32; 4],
    p0p2: [f32; 4],
    width: f32,
    height: f32,
    _a: f32,
    _b: f32,
}

pub struct Camera {
    pub uniforms: CameraUniforms,
    position: Vec3,
    rotation: Vec3,
    forward: Vec3,
    right: Vec3,
}

impl Camera {
    pub fn new(width: u32, height: u32, position: Vec3, rotation: Vec3) -> Self {
        let (forward, right, p0, p0p1, p0p2) = calc_vecs(position, rotation);

        Self {
            position,
            rotation,
            forward,
            right,
            uniforms: CameraUniforms {
                pos: position.xyzx().to_array(),
                p0: p0.xyzx().to_array(),
                p0p1: p0p1.xyzx().to_array(),
                p0p2: p0p2.xyzx().to_array(),
                width: width as f32,
                height: height as f32,
                _a: 0.0,
                _b: 0.0,
            },
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    virtual_keycode, ..
                },
                ..
            } => match virtual_keycode {
                Some(VirtualKeyCode::W) => {
                    self.position += self.forward * 0.01;
                    true
                }
                Some(VirtualKeyCode::S) => {
                    self.position -= self.forward * 0.01;
                    true
                }
                Some(VirtualKeyCode::A) => {
                    self.position -= self.right * 0.01;
                    true
                }
                Some(VirtualKeyCode::D) => {
                    self.position += self.right * 0.01;
                    true
                }
                Some(VirtualKeyCode::R) => {
                    self.position += Vec3::new(0.0, 0.01, 0.0);
                    true
                }
                Some(VirtualKeyCode::F) => {
                    self.position -= Vec3::new(0.0, 0.01, 0.0);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.uniforms.width = width as f32;
        self.uniforms.height = height as f32;
    }

    pub fn update(&mut self) {
        let (forward, right, p0, p0p1, p0p2) = calc_vecs(self.position, self.rotation);
        self.forward = forward;
        self.right = right;
        self.uniforms.p0 = p0.xyzx().to_array();
        self.uniforms.p0p1 = p0p1.xyzx().to_array();
        self.uniforms.p0p2 = p0p2.xyzx().to_array();
    }

    pub fn render_ui(&mut self, ui: &Ui) {
        Window::new("camera")
            .size([250.0, 100.0], Condition::Always)
            .build(ui, || {
                ui.text(format!("Width: {}", self.uniforms.width));
                ui.text(format!("Height: {}", self.uniforms.height));

                ui.text(format!(
                    "pos: {} {} {}",
                    self.position.x, self.position.y, self.position.z
                ));
                ui.text(format!(
                    "rot: {} {} {}",
                    self.rotation.x, self.rotation.y, self.rotation.z
                ));
            });
    }
}

fn calc_vecs(position: Vec3, rotation: Vec3) -> (Vec3, Vec3, Vec3, Vec3, Vec3) {
    let forward = -Vec3::normalize(Vec3::new(rotation.x.sin(), rotation.x, rotation.y.cos()));

    let right = Vec3::normalize(Vec3::cross(forward, Vec3::new(0.0, 1.0, 0.0)));

    let up = Vec3::normalize(Vec3::cross(right, forward));

    let center = position + forward;
    let p0 = center - up - right;
    let p1 = center - up + right;
    let p2 = center + up - right;

    (forward, right, p0, p1 - p0, p2 - p0)
}

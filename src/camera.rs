use imgui::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    width: u32,
    height: u32,
}

pub struct Camera {
    pub uniforms: CameraUniforms,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            uniforms: CameraUniforms { width, height },
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        false
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.uniforms.width = width;
        self.uniforms.height = height;
    }

    pub fn render_ui(&mut self, ui: &Ui) {
        Window::new("camera")
            .size([200.0, 100.0], Condition::Always)
            .build(ui, || {
                ui.text(format!("Width: {}", self.uniforms.width));
                ui.text(format!("Height: {}", self.uniforms.height));
            });
    }
}

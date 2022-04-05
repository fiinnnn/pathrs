use imgui::*;

pub struct CameraController {}

impl CameraController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(
        &self,
        camera: &mut pathrs_shared::Camera,
        input: &winit_input_helper::WinitInputHelper,
        window: &winit::window::Window,
    ) {
    }
}

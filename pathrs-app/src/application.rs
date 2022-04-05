use std::time::Instant;

use futures::executor::block_on;
use game_loop::game_loop;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use crate::camera_controller::CameraController;
use crate::renderer::Renderer;

struct WGPUBackend {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl WGPUBackend {
    async fn init(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let window_size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        surface.configure(&device, &surface_config);

        Self {
            surface,
            surface_config,
            _adapter: adapter,
            device,
            queue,
        }
    }
}

pub struct Application {
    wgpu_backend: WGPUBackend,
    input: WinitInputHelper,
    imgui_ctx: imgui::Context,
    imgui_platform: imgui_winit_support::WinitPlatform,
    renderer: Renderer,
    camera: pathrs_shared::Camera,
    camera_controller: CameraController,
    puffin_profiler_ui: puffin_imgui::ProfilerUi,
    last_frame: Instant,
}

impl Application {
    pub async fn new(window: &Window) -> Self {
        let wgpu_backend = WGPUBackend::init(window).await;
        let window_size = window.inner_size();

        let input = WinitInputHelper::new();

        let mut imgui_ctx = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);
        imgui_platform.attach_window(
            imgui_ctx.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Locked(1.0),
        );

        imgui_ctx.set_ini_filename(None);

        imgui_ctx
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: 13.0,
                    ..Default::default()
                }),
            }]);

        let renderer = Renderer::new(
            &wgpu_backend.device,
            &wgpu_backend.queue,
            &mut imgui_ctx,
            &wgpu_backend.surface_config,
            window_size.width,
            window_size.height,
        );

        let camera = pathrs_shared::Camera::new(
            window_size.width as f32,
            window_size.height as f32,
            // Vec3::new(0.0, 0.0, 0.0),
            // Vec3::new(0.0, 0.0, 0.0),
        );

        let camera_controller = CameraController::new();

        let puffin_profiler_ui = puffin_imgui::ProfilerUi::default();

        let last_frame = Instant::now();

        Self {
            wgpu_backend,
            input,
            imgui_ctx,
            imgui_platform,
            renderer,
            camera,
            camera_controller,
            puffin_profiler_ui,
            last_frame,
        }
    }

    fn resize(&mut self, window: &Window, width: u32, height: u32) {
        self.wgpu_backend.surface_config.width = width;
        self.wgpu_backend.surface_config.height = height;
        self.wgpu_backend
            .surface
            .configure(&self.wgpu_backend.device, &self.wgpu_backend.surface_config);

        self.renderer
            .resize(&self.wgpu_backend.device, width, height);

        self.camera.width = width as f32;
        self.camera.height = height as f32;

        self.camera.position = glam::Vec4::new(0.0, 0.0, 1.0, 0.0);

        self.render(window);
    }

    fn handle_event<T>(&mut self, event: &winit::event::Event<T>, window: &Window) {
        let mut handled = false;

        self.imgui_platform
            .handle_event(self.imgui_ctx.io_mut(), window, event);

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    self.resize(window, size.width, size.height);
                    handled = true;
                }
                _ => (),
            },
            _ => (),
        }

        if handled {
            return;
        }

        self.input.update(event);
    }

    fn update(&mut self, window: &Window) {
        self.camera_controller
            .update(&mut self.camera, &self.input, window);
    }

    fn render(&mut self, window: &Window) {
        puffin::profile_function!();

        let now = Instant::now();
        let dt = now - self.last_frame;
        self.last_frame = now;

        self.imgui_ctx.io_mut().update_delta_time(dt);

        let texture = match self.wgpu_backend.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Unable to acquire texture surface texture: {}", e);
                return;
            }
        };

        self.imgui_platform
            .prepare_frame(self.imgui_ctx.io_mut(), window)
            .expect("Failed to prepare imgui frame");

        let imgui_frame = self.imgui_ctx.frame();

        //self.puffin_profiler_ui.window(&imgui_frame);

        //self.camera_controller.render_ui(&imgui_frame);

        self.renderer.render(
            &self.wgpu_backend.device,
            &texture,
            imgui_frame,
            &self.wgpu_backend.queue,
            &self.camera,
        );

        texture.present();
    }
}

pub fn run_app(title: &str, width: u32, height: u32) -> ! {
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title(title)
        .with_inner_size(winit::dpi::PhysicalSize { width, height })
        .build(&event_loop)
        .unwrap();

    let app = block_on(Application::new(&window));

    game_loop(
        event_loop,
        window,
        app,
        60,
        0.1,
        |g| {
            g.game.update(&g.window);
        },
        |g| {
            g.game.render(&g.window);
            puffin::GlobalProfiler::lock().new_frame();
        },
        |g, event| {
            let mut handled = false;
            match &event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        g.exit();
                        handled = true;
                    }
                    _ => (),
                },
                _ => (),
            };

            if !handled {
                g.game.handle_event(&event, &g.window);
            }
        },
    );
}

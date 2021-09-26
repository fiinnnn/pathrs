use std::time::Instant;

use futures::executor::block_on;
use game_loop::game_loop;
use winit::window::Window;

use crate::camera::Camera;
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
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
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
    imgui_ctx: imgui::Context,
    imgui_platform: imgui_winit_support::WinitPlatform,
    renderer: Renderer,
    camera: Camera,
    puffin_profiler_ui: puffin_imgui::ProfilerUi,
    last_frame: Instant,
}

impl Application {
    pub async fn new(window: &Window) -> Self {
        let wgpu_backend = WGPUBackend::init(window).await;
        let window_size = window.inner_size();

        let mut imgui_ctx = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);
        imgui_platform.attach_window(
            imgui_ctx.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        imgui_ctx.set_ini_filename(None);

        let font_size = (13.0 * window.scale_factor()) as f32;
        imgui_ctx.io_mut().font_global_scale = (1.0 / window.scale_factor()) as f32;

        imgui_ctx.fonts().add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
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
        let camera = Camera::new(window_size.width, window_size.height);

        let puffin_profiler_ui = puffin_imgui::ProfilerUi::default();

        let last_frame = Instant::now();

        Self {
            wgpu_backend,
            imgui_ctx,
            imgui_platform,
            renderer,
            camera,
            puffin_profiler_ui,
            last_frame,
        }
    }

    fn resize(&mut self, window: &winit::window::Window, width: u32, height: u32) {
        self.wgpu_backend.surface_config.width = width;
        self.wgpu_backend.surface_config.height = height;
        self.wgpu_backend
            .surface
            .configure(&self.wgpu_backend.device, &self.wgpu_backend.surface_config);

        self.renderer
            .resize(&self.wgpu_backend.device, width, height);

        self.camera.resize(width, height);

        self.render(window);
    }

    fn handle_event(&mut self, event: &winit::event::Event<()>, window: &winit::window::Window) {
        let mut handled = false;

        self.imgui_platform.handle_event(self.imgui_ctx.io_mut(), window, event);

        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                match event {
                    winit::event::WindowEvent::Resized(size) => {
                        self.resize(window, size.width, size.height);
                        handled = true;
                    },
                    _ => (),
                }
            }
            _ => (),
        }

        if handled {
            return;
        }
    }

    fn render(&mut self, window: &winit::window::Window) {
        puffin::profile_function!();

        let now = Instant::now();
        let dt = now - self.last_frame;
        self.last_frame = now;

        self.imgui_ctx.io_mut().update_delta_time(dt);

        let frame = match self
            .wgpu_backend
            .surface
            .get_current_frame()
        {
            Ok(frame) => frame,
            Err(e) => {
                log::error!("Dropped frame: {}", e);
                return;
            }
        };

        self.imgui_platform
            .prepare_frame(self.imgui_ctx.io_mut(), window)
            .expect("Failed to prepare imgui frame");

        let imgui_frame = self.imgui_ctx.frame();


        self.puffin_profiler_ui.window(&imgui_frame);

        self.renderer.render(
            &self.wgpu_backend.device,
            &frame,
            imgui_frame,
            &self.wgpu_backend.queue,
            &self.camera,
        );
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
        |_g| {},
        |g| {
            g.game.render(&g.window);
            puffin::GlobalProfiler::lock().new_frame();
        },
        |g, event| {
            let mut handled = false;
            match &event {
                winit::event::Event::WindowEvent { event, .. } => {

                    match event {
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
                    }
                }
                _ => (),
            };

            if !handled {
                g.game.handle_event(&event, &g.window);
            }
        },
    );
}

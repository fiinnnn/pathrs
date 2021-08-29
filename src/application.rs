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
            present_mode: wgpu::PresentMode::Mailbox,
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
    renderer: Renderer,
    camera: Camera,
}

impl Application {
    pub async fn new(window: &Window) -> Self {
        let wgpu_backend = WGPUBackend::init(window).await;
        let window_size = window.inner_size();
        let renderer = Renderer::new(
            &wgpu_backend.device,
            &wgpu_backend.surface_config,
            window_size.width,
            window_size.height,
        );
        let camera = Camera::new(window_size.width, window_size.height);

        Self {
            wgpu_backend,
            renderer,
            camera,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.wgpu_backend.surface_config.width = width;
        self.wgpu_backend.surface_config.height = height;
        self.wgpu_backend
            .surface
            .configure(&self.wgpu_backend.device, &self.wgpu_backend.surface_config);

        self.renderer
            .resize(&self.wgpu_backend.device, width, height);

        self.camera.resize(width, height);

        self.render();
    }

    fn handle_window_event(&mut self, event: &winit::event::WindowEvent) {
        let mut handled = false;

        match event {
            winit::event::WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
                handled = true;
            }
            _ => (),
        }

        if handled {
            return;
        }

        self.camera.input(event);
    }

    fn render(&mut self) {
        let frame = self
            .wgpu_backend
            .surface
            .get_current_frame()
            .expect("Unable to get current frame");
        self.renderer.render(
            &self.wgpu_backend.device,
            &frame,
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
        |g| g.game.render(),
        |g, event| match event {
            winit::event::Event::WindowEvent { event, .. } => {
                let mut handled = false;
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
                if !handled {
                    g.game.handle_window_event(&event)
                }
            }
            _ => (),
        },
    );
}

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};

use crate::camera::Camera;
use crate::renderer::Renderer;

struct SDLBackend {
    context: sdl2::Sdl,
    window: sdl2::video::Window,
}

impl SDLBackend {
    fn init(title: &str, width: u32, height: u32) -> Self {
        let context = sdl2::init().unwrap();

        let video = context.video().unwrap();

        let window = video
            .window(title, width, height)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        Self { context, window }
    }
}

struct WGPUBackend {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

impl WGPUBackend {
    async fn init(window: &sdl2::video::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

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
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let window_size = window.size();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window_size.0,
            height: window_size.1,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
        }
    }
}

pub struct Application {
    sdl_backend: SDLBackend,
    wgpu_backend: WGPUBackend,
    renderer: Renderer,
    camera: Camera,
}

impl Application {
    pub async fn new(title: &str, width: u32, height: u32) -> Self {
        let sdl_backend = SDLBackend::init(title, width, height);
        let wgpu_backend = WGPUBackend::init(&sdl_backend.window).await;
        let renderer = Renderer::new(&wgpu_backend.device, &wgpu_backend.sc_desc, width, height);
        let camera = Camera::new(width, height);

        Self {
            sdl_backend,
            wgpu_backend,
            renderer,
            camera,
        }
    }

    pub fn run(&mut self) {
        let mut event_pump = self.sdl_backend.context.event_pump().unwrap();

        let mut running = true;
        while running {
            for event in event_pump.poll_iter() {
                match self.camera.input(&event) {
                    true => continue,
                    _ => {}
                }

                match event {
                    Event::Quit { .. } => running = false,
                    Event::KeyUp {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => running = false,
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        self.resize(width as u32, height as u32);
                    }
                    _ => {}
                }
            }

            self.render();
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.wgpu_backend.sc_desc.width = width;
        self.wgpu_backend.sc_desc.height = height;
        self.wgpu_backend.swap_chain = self
            .wgpu_backend
            .device
            .create_swap_chain(&self.wgpu_backend.surface, &self.wgpu_backend.sc_desc);

        self.renderer
            .resize(&self.wgpu_backend.device, width, height);

        self.camera.resize(width, height);

        self.render();
    }

    fn render(&mut self) {
        let frame = self
            .wgpu_backend
            .swap_chain
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

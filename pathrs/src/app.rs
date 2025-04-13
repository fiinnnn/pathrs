use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticPath, Diagnostics, FrameTimeDiagnosticsPlugin, RegisterDiagnostic,
    },
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

use crate::ui::{EguiViewport, init_ui, render_ui};
use bevy_egui::{EguiContexts, EguiPlugin};
use crossbeam_channel::Sender;
use pathrs_renderer::{RenderResult, RenderSystem, RendererCmd, renderer::CPURenderer};

pub fn run_bevy_app() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EguiPlugin)
        .register_diagnostic(Diagnostic::new(RENDER_TIME).with_suffix(" ms"))
        .register_diagnostic(Diagnostic::new(RAYS_PER_SECOND).with_suffix(" R/s"))
        .add_systems(Startup, (init_ui, init_renderer))
        .add_systems(
            Update,
            (render_ui, resize_render_target, receive_render).chain(),
        )
        .run();
}

pub const RENDER_TIME: DiagnosticPath = DiagnosticPath::const_new("render_time");
pub const RAYS_PER_SECOND: DiagnosticPath = DiagnosticPath::const_new("rays_per_second");

#[derive(Resource)]
struct RenderTarget {
    image_handle: Handle<Image>,
    size: UVec2,
}

#[derive(Resource)]
struct RenderTask {
    cmd_tx: Sender<RendererCmd>,
    output: triple_buffer::Output<RenderResult>,
}

impl Drop for RenderTask {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(RendererCmd::Stop);
    }
}

fn init_renderer(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut contexts: EguiContexts,
) {
    let size = Extent3d {
        width: 100,
        height: 100,
        ..Default::default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..Default::default()
    };

    image.resize(size);

    let image_handle = images.add(image);
    let size = UVec2::new(size.width, size.height);
    let egui_viewport_texture = bevy_egui::egui::load::SizedTexture {
        id: contexts.add_image(image_handle.clone_weak()),
        size: bevy_egui::egui::vec2(size.x as f32, size.y as f32),
    };

    commands.insert_resource(RenderTarget { image_handle, size });
    commands.insert_resource(EguiViewport {
        texture: egui_viewport_texture,
        size,
    });

    let (renderer, cmd_tx, out) = RenderSystem::<CPURenderer>::new(size.x, size.y);

    renderer.start_thread();

    commands.insert_resource(RenderTask {
        cmd_tx,
        output: out,
    });
}

fn resize_render_target(
    render_task: Res<RenderTask>,
    mut render_target: ResMut<RenderTarget>,
    egui_viewport: Res<EguiViewport>,
    mut last_size: Local<UVec2>,
) {
    if egui_viewport.size != render_target.size
        && egui_viewport.size == *last_size
        && egui_viewport.size.x > 0
        && egui_viewport.size.y > 0
    {
        println!("resize {} {}", egui_viewport.size.x, egui_viewport.size.y);
        render_target.size = egui_viewport.size;

        _ = render_task.cmd_tx.send(RendererCmd::Resize {
            width: egui_viewport.size.x,
            height: egui_viewport.size.y,
        });
    }

    *last_size = egui_viewport.size;
}

fn receive_render(
    mut render_task: ResMut<RenderTask>,
    render_target: Res<RenderTarget>,
    mut egui_viewport: ResMut<EguiViewport>,
    mut images: ResMut<Assets<Image>>,
    mut diagnostics: Diagnostics,
) {
    let RenderResult {
        image_data,
        image_size,
        render_time,
        rays_per_second,
    } = render_task.output.read();

    if image_size.x == 0 || image_size.y == 0 {
        return;
    }

    diagnostics.add_measurement(&RENDER_TIME, || render_time.as_millis() as f64);
    diagnostics.add_measurement(&RAYS_PER_SECOND, || *rays_per_second);

    let image = images.get_mut(&render_target.image_handle).unwrap();

    image.resize(Extent3d {
        width: image_size.x,
        height: image_size.y,
        ..Default::default()
    });
    egui_viewport.texture.size = bevy_egui::egui::vec2(image_size.x as f32, image_size.y as f32);

    let image_bytes = image_data
        .iter()
        .flatten()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    image.data = image_bytes;
}

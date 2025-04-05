use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::uvec2,
    prelude::*,
};
use bevy_egui::{EguiContexts, egui};
use egui_tiles::{Container, Linear, LinearDir, Tile, TileId, Tiles, Tree, UiResponse};

use crate::{RAYS_PER_SECOND, RENDER_TIME};

#[derive(Resource)]
pub struct UiState {
    tree: Tree<Pane>,
}

#[derive(Resource)]
pub struct EguiViewport {
    pub texture: egui::load::SizedTexture,
    pub size: UVec2,
}

enum Pane {
    Viewport,
    Performance,
}

pub fn init_ui(mut commands: Commands) {
    let mut tiles = Tiles::default();
    let panes = [
        tiles.insert_pane(Pane::Viewport),
        tiles.insert_pane(Pane::Performance),
    ];
    let container = Tile::Container(Container::Linear(Linear::new_binary(
        LinearDir::Horizontal,
        panes,
        0.75,
    )));
    let root = tiles.insert_new(container);

    let tree = Tree::new("tree", root, tiles);

    commands.insert_resource(UiState { tree });
}

pub fn render_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut viewport: ResMut<EguiViewport>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut tab_behavior = TabBehavior {
        viewport: &mut viewport,
        diagnostics: &diagnostics,
    };

    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui_state.tree.ui(&mut tab_behavior, ui)
    });
}

struct TabBehavior<'a> {
    viewport: &'a mut EguiViewport,
    diagnostics: &'a DiagnosticsStore,
}

impl egui_tiles::Behavior<Pane> for TabBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Viewport => "Viewport".into(),
            Pane::Performance => "Performance".into(),
        }
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, pane: &mut Pane) -> UiResponse {
        match pane {
            Pane::Viewport => {
                let available_size = ui.available_size();
                self.viewport.size = uvec2(
                    available_size.x.ceil() as u32,
                    available_size.y.ceil() as u32,
                );

                let image = egui::Image::from_texture(self.viewport.texture);

                ui.add(image);
            }
            Pane::Performance => {
                if let Some(fps) = self
                    .diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(Diagnostic::smoothed)
                {
                    ui.label(format!("FPS: {fps:.2}"));
                }

                if let Some(frame_time) = self
                    .diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                    .and_then(Diagnostic::smoothed)
                {
                    ui.label(format!("frame-time: {frame_time:#.2} ms"));
                }

                if let Some(render_time) = self
                    .diagnostics
                    .get(&RENDER_TIME)
                    .and_then(Diagnostic::smoothed)
                {
                    ui.label(format!("render-time: {render_time:#.2} ms"));
                }

                if let Some(mut rays_per_second) = self
                    .diagnostics
                    .get(&RAYS_PER_SECOND)
                    .and_then(Diagnostic::smoothed)
                {
                    let unit = if rays_per_second > 1_000_000.0 {
                        rays_per_second /= 1_000_000.0;
                        "M"
                    } else if rays_per_second > 1_000.0 {
                        rays_per_second /= 1_000.0;
                        "k"
                    } else {
                        ""
                    };

                    ui.label(format!("rays: {rays_per_second:#.2} {unit}Ray/s"));
                }
            }
        }

        Default::default()
    }
}

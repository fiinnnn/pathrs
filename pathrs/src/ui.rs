use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::uvec2,
    prelude::*,
};
use bevy_egui::{EguiContexts, egui};
use egui_tiles::{Container, Linear, LinearDir, Tile, TileId, Tiles, Tree, UiResponse};
use pathrs_renderer::metrics::RendererMetrics;

use crate::app::RenderTask;

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
    render_task: Res<RenderTask>,
) {
    let mut tab_behavior = TabBehavior {
        viewport: &mut viewport,
        diagnostics: &diagnostics,
        renderer_metrics: &render_task.metrics,
    };

    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui_state.tree.ui(&mut tab_behavior, ui)
    });
}

struct TabBehavior<'a> {
    viewport: &'a mut EguiViewport,
    diagnostics: &'a DiagnosticsStore,
    renderer_metrics: &'a RendererMetrics,
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

                let passes = self.renderer_metrics.capacity;
                ui.label("Render time:");
                {
                    use egui_plot::{HLine, Line, Plot, PlotPoints};

                    let points = PlotPoints::from_ys_f32(
                        &self.renderer_metrics.render_times().collect::<Vec<_>>(),
                    );
                    let line = Line::new("Time (ms)", points)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_RED));

                    let avg = self.renderer_metrics.render_times().sum::<f32>() / passes as f32;
                    let avg_line = HLine::new("average", avg)
                        .style(egui_plot::LineStyle::Dotted { spacing: 3.0 })
                        .stroke(egui::Stroke::new(0.5, egui::Color32::LIGHT_GRAY));

                    let avg_label = egui_plot::Text::new(
                        "average",
                        egui_plot::PlotPoint::new((passes - 2) as f64, avg as f64),
                        format!("{avg:.2} ms"),
                    )
                    .anchor(egui::Align2::RIGHT_BOTTOM)
                    .color(egui::Color32::LIGHT_GRAY);

                    Plot::new("Render Time Plot")
                        .height(150.0)
                        .width(ui.available_width())
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .show_grid(false)
                        .set_margin_fraction(egui::vec2(0.0, 0.0))
                        .include_y(0.0)
                        .include_y(400.0)
                        .include_x(0.0)
                        .include_x(passes as f64)
                        .show_axes([false, false])
                        .show(ui, |plot_ui| {
                            plot_ui.line(line);
                            plot_ui.hline(avg_line);
                            plot_ui.text(avg_label);
                        });
                }

                ui.label("Rays per second:");
                {
                    use egui_plot::{Line, Plot, PlotPoints};

                    let rays = self
                        .renderer_metrics
                        .rays_per_second()
                        .collect::<Vec<f32>>();

                    let points = PlotPoints::from_ys_f32(&rays);
                    let line = Line::new("Rays/s", points)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE));

                    let avg = self.renderer_metrics.rays_per_second().sum::<f32>() / passes as f32;
                    let avg_line = egui_plot::HLine::new("average", avg)
                        .style(egui_plot::LineStyle::Dotted { spacing: 3.0 })
                        .stroke(egui::Stroke::new(0.5, egui::Color32::LIGHT_GRAY));

                    let avg_label = egui_plot::Text::new(
                        "average",
                        egui_plot::PlotPoint::new((passes - 2) as f64, avg as f64),
                        format!(
                            "{:.2} {}Ray/s",
                            if avg >= 1_000_000.0 {
                                avg / 1_000_000.0
                            } else if avg >= 1_000.0 {
                                avg / 1_000.0
                            } else {
                                avg
                            },
                            if avg >= 1_000_000.0 {
                                "M"
                            } else if avg >= 1_000.0 {
                                "k"
                            } else {
                                ""
                            }
                        ),
                    )
                    .anchor(egui::Align2::RIGHT_BOTTOM)
                    .color(egui::Color32::LIGHT_GRAY);

                    Plot::new("Rays per Second")
                        .height(150.0)
                        .width(ui.available_width())
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .show_grid(false)
                        .set_margin_fraction(egui::vec2(0.0, 0.0))
                        .include_y(0.0)
                        .include_y(20000000.0)
                        .include_x(0.0)
                        .include_x(passes as f64)
                        .show_axes([false, false])
                        .show(ui, |plot_ui| {
                            plot_ui.line(line);
                            plot_ui.hline(avg_line);
                            plot_ui.text(avg_label);
                        });
                }

                ui.label("Ray depths:");
                let histogram = self.renderer_metrics.average_depth_histogram();
                {
                    use egui_plot::{Bar, BarChart, Plot};

                    let bars: Vec<Bar> = histogram
                        .iter()
                        .enumerate()
                        .map(|(i, &value)| Bar::new(i as f64, value as f64).name(format!("d{}", i)))
                        .collect();

                    let chart = BarChart::new("Ray depth", bars).color(egui::Color32::LIGHT_BLUE);

                    Plot::new("depth_histogram_plot")
                        .height(75.0)
                        .width(ui.available_width())
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .show_grid(false)
                        .show_background(false)
                        .set_margin_fraction(egui::vec2(0.0, 0.0))
                        .include_y(0.0)
                        .include_y(1.0)
                        .include_x(0.0)
                        .include_x(10.0)
                        .show_axes([false, false])
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(chart);
                        });
                }
            }
        }

        Default::default()
    }
}

use clap::{Parser, Subcommand};
use pathrs_renderer::Renderer;
use ppm::write_ppm_file;

mod app;
mod ppm;
mod ui;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Subcommand)]
enum Command {
    Run,
    RenderImage {
        width: u32,
        height: u32,
        samples_per_pixel: u32,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Run => app::run_bevy_app(),
        Command::RenderImage {
            width,
            height,
            samples_per_pixel,
        } => render_image(width, height, samples_per_pixel),
    }
}

fn render_image(width: u32, height: u32, samples_per_pixel: u32) {
    let (renderer, _, _) = Renderer::new(width, height);

    let img = renderer.render_image(samples_per_pixel);

    if let Err(err) = write_ppm_file(&img, width, height) {
        eprintln!("Error writing ppm file: {err}");
    }
}

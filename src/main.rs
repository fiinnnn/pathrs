mod application;
mod camera;
mod renderer;

use application::run_app;

fn main() {
    env_logger::init();

    run_app("Pathrs", 1280, 720);
}

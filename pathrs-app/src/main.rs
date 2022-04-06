mod application;
mod camera;
mod renderer;

use application::run_app;

fn main() {
    puffin::set_scopes_on(true);
    env_logger::init();

    run_app("Pathrs", 1280, 720);
}

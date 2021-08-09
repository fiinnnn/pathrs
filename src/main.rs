use futures::executor::block_on;

mod application;
mod renderer;
mod camera;

use application::Application;

fn main() {
    env_logger::init();

    let mut app = block_on(Application::new("Pathrs", 1280, 720));

    app.run();
}

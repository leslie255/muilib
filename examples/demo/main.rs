use std::path::PathBuf;

use winit::event_loop::EventLoop;

use muilib::AppResources;

use crate::app::Application;

mod theme;
mod app;

fn main() {
    env_logger::init();

    // TODO: read resource directory path from command line args.
    let resources = AppResources::new(PathBuf::from("res/"));
    let event_loop = EventLoop::builder().build().unwrap();
    event_loop
        .run_app(&mut Application::new(&resources))
        .unwrap();
}

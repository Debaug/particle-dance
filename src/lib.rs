use std::time::Duration;

use color_eyre::Result;
use glam::vec4;

use app::App;
use dance::DanceSubAppBuilder;
use log::LogSubApp;

pub mod app;
pub mod dance;
pub mod data;
pub mod log;

pub fn run() -> Result<()> {
    env_logger::init();
    App::new(
        Duration::from_millis(10),
        winit::window::WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 1000)),
    )
    .add_sub_app(LogSubApp)
    .add_sub_app(DanceSubAppBuilder {
        n_points: 1_000_000,
        transformation_colors: vec![
            vec4(0.9, 0.9, 0.6, 1.0),
            vec4(0.6, 0.9, 0.9, 1.0),
            vec4(0.9, 0.6, 0.9, 1.0),
            vec4(0.9, 0.6, 0.4, 1.0),
        ],
    })
    .run()
}

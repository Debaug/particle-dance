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
pub mod random;
pub mod time;

pub fn run() -> Result<()> {
    env_logger::init();
    App::new(
        Duration::from_millis(10),
        winit::window::WindowAttributes::default()
            .with_inner_size(winit::dpi::PhysicalSize::new(1080, 1080)),
    )
    .add_sub_app(LogSubApp)
    .add_sub_app(DanceSubAppBuilder {
        n_points: 2_000_000,
        transformation_colors: vec![
            vec4(0.9, 0.9, 0.6, 1.0),
            vec4(0.6, 0.9, 0.9, 1.0),
            vec4(0.9, 0.6, 0.9, 1.0),
            vec4(0.9, 0.6, 0.4, 1.0),
            vec4(0.4, 0.6, 0.9, 1.0),
        ],
    })
    .run()
}

#[cfg(target_arch = "wasm32")]
pub fn run_web(canvas: web_sys::HtmlCanvasElement) {
    use winit::platform::web::WindowAttributesExtWebSys;

    env_logger::init();
    let _ = App::new(
        Duration::from_millis(10),
        winit::window::WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 1000))
            .with_canvas(Some(canvas)),
    )
    .add_sub_app(LogSubApp)
    .add_sub_app(DanceSubAppBuilder {
        n_points: 2_000_000,
        transformation_colors: vec![
            vec4(0.9, 0.9, 0.6, 1.0),
            vec4(0.6, 0.9, 0.9, 1.0),
            vec4(0.9, 0.6, 0.9, 1.0),
            vec4(0.9, 0.6, 0.4, 1.0),
            vec4(0.4, 0.6, 0.9, 1.0),
        ],
    })
    .run();
}

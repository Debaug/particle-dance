use std::time::Duration;

use color_eyre::Result;

use app::App;
use dance::DanceSubAppBuilder;
use log::LogSubApp;

pub mod app;
pub mod dance;
pub mod data;
pub mod log;

pub fn run() -> Result<()> {
    env_logger::init();
    App::new(Duration::from_millis(100))
        // .add_sub_app(LogSubApp)
        .add_sub_app(DanceSubAppBuilder {
            n_points: 1_000_000,
        })
        .run()
}

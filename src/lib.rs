use std::time::Duration;

use color_eyre::Result;

use crate::app::App;
use crate::log::LogSubApp;

pub mod app;
pub mod log;

pub fn run() -> Result<()> {
    env_logger::init();
    App::new(Duration::from_millis(1000))
        .add_sub_app(LogSubApp)
        .run()
}

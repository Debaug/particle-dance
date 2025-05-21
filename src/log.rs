use log::info;

use crate::app::{SubApp, SubAppBuilder};

pub struct LogSubApp;

impl SubApp for LogSubApp {
    fn update(
        &mut self,
        _context: &crate::app::Context,
        delta_time: std::time::Duration,
    ) -> color_eyre::eyre::Result<()> {
        info!("delta = {}ms", delta_time.as_millis());
        Ok(())
    }
}

impl SubAppBuilder for LogSubApp {
    fn build(
        self: Box<Self>,
        _context: &crate::app::Context,
    ) -> color_eyre::eyre::Result<Box<dyn SubApp>> {
        Ok(self)
    }
}

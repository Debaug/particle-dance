use log::info;

use crate::app::{SubApp, SubAppBuilder, Time};

pub struct LogSubApp;

impl SubApp for LogSubApp {
    fn update(
        &mut self,
        _context: &crate::app::Context,
        time: Time,
    ) -> color_eyre::eyre::Result<()> {
        info!(
            "elapsed = {}s, delta = {}ms",
            time.elapsed_f32 as u32,
            time.delta.as_millis()
        );
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

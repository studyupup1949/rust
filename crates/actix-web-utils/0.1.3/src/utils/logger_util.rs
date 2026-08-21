use log::{SetLoggerError, LevelFilter};

use crate::extensions::logger::SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}
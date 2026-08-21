use log::{SetLoggerError, LevelFilter};

use crate::extensions::logger::SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init(max_log_level_filter: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(max_log_level_filter))
}
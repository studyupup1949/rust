use log::{SetLoggerError, LevelFilter};

use crate::extensions::logger::SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_logger_default() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}
/// Made this get an integer so that you don't have to import the Log library in your code too, as the LevelFilter enum is inside it.
/// 0 -> Off, 1 -> Trace, 2 -> Debug, 3 -> Info, 4 -> Warn, 5 -> Error
pub fn init_logger_custom(max_log_level_filter: i8) -> Result<(), SetLoggerError> {
    let level_filter: LevelFilter = match max_log_level_filter {
        0 => LevelFilter::Off,
        1 => LevelFilter::Trace,
        2 => LevelFilter::Debug,
        3 => LevelFilter::Info,
        4 => LevelFilter::Warn,
        5 => LevelFilter::Error,
        _ => panic!("INVALID max_log_level_filter. Only values between 0-5 are allowed."),
    };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level_filter))
}
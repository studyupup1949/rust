#[cfg(feature = "trace")]
use crate::fs::get_exe_dir;
#[cfg(feature = "trace")]
use crate::time::LocalTimeFormatter;
#[cfg(feature = "trace")]
use std::io;
#[cfg(feature = "trace")]
use tracing::*;
use tracing_appender::non_blocking::NonBlocking;

#[cfg(feature = "trace")]
#[macro_export]
macro_rules! init_tracing {
    ($e:expr) => {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", "poem=debug");
        }
        let log_path = format!("{}/logs", get_exe_dir());
        let file_appender = tracing_appender::rolling::daily(log_path, $e);
        let format = tracing_subscriber::fmt::format()
            .with_level(true)
            .with_target(true)
            .with_timer(LocalTimeFormatter);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .with_writer(std::io::stdout)
            .with_writer(non_blocking)
            .with_ansi(true)
            .event_format(format)
            .init();
    };
}

#[cfg(feature = "trace")]
#[macro_export]
macro_rules! init_global_tracing {
       ($d:expr,$f:expr,$w:expr) => {
        thread::spawn(|| {
            let log_appender =tracing_appender::rolling::daily($d, $f);
            let format = tracing_subscriber::fmt::format()
                .with_level(true)
                .with_target(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_timer(LocalTimeFormatter);
            let (non_blocking, _guard) = tracing_appender::non_blocking(log_appender);
            let mut loggerBuilder = tracing_subscriber::fmt()
                .with_max_level(LevelFilter::TRACE)
                .with_writer(std::io::stdout)
                .with_writer(non_blocking)
                .with_ansi(true)
                .event_format(format);
            //if $w.is_some() {
               //loggerBuilder = loggerBuilder.with_writer($w.unwrap());
            //}
            let logger = loggerBuilder.finish();
            tracing::subscriber::set_global_default(logger);
            loop {
                thread::sleep(time::Duration::from_secs(300 as u64));
                info!("LogThread is running.");
            }
        });

        std::panic::set_hook(Box::new(|panic| {
            if let Some(location) = panic.location() {
                error!(
                    message = %panic,
                    panic.file = location.file(),
                    panic.line = location.line(),
                    panic.column = location.column(),
                );
            } else {
                error!(message = %panic);
            }
        }));
    }
}

#[cfg(test)]
#[cfg(feature = "trace")]
mod tests {
    use super::*;
    use crate::trace::metadata::LevelFilter;

    #[test]
    fn test_init_log() {
        init_tracing!("Seoul");
    }
}

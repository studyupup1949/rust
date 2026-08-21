use fern::colors::{Color, ColoredLevelConfig};

pub fn init() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::BrightMagenta)
        .trace(Color::White);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}{time} {level}{color_line} -> [{target}] \x1B[0m{message}",
                color_line = format_args!("\x1B[{}m", Color::BrightBlack.to_fg_str()),
                time = chrono::Local::now().format("%H:%M:%S"),
                level = colors.color(record.level()),
                target = record.target(),
                message = message
            ))
        })
        .level(log::LevelFilter::max())
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

// declare modules
mod config;
mod download;
mod form;
mod version;

#[tokio::main]
async fn main() {
    // initialize configuration
    let _ = config::CONFIG;

    // import warp items
    use warp::{Filter, body, get, path, post, serve};

    // define routes
    let routes = get()
        .map(form::render_form)
        .or(post()
            .and(body::content_length_limit(config::CONFIG.max_body_size))
            .and(body::form())
            .and(path("dl"))
            .and_then(download::handle_download))
        .or(get().and(path("version")).map(version::plain));

    // initialize logger
    init_logger();

    // print version and bind address
    log::info!("Running aa-fastlink {}", env!("CARGO_PKG_VERSION"));
    log::info!(
        "Binding to http://{}:{}",
        config::CONFIG.bind_ip,
        config::CONFIG.bind_port
    );

    // start server
    serve(routes)
        .run((config::CONFIG.bind_ip, config::CONFIG.bind_port))
        .await;
}

// initializes logger with custom settings
pub fn init_logger() {
    use std::io::Write;

    let mut builder = env_logger::Builder::new();

    builder.format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()));

    builder.filter_level(if config::CONFIG.debug_logging {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    });

    builder.init();
}

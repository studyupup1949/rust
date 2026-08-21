mod config;
mod download;
mod form;

#[tokio::main]
async fn main() {
    // import warp items
    use warp::{Filter, body, get, path, post, serve};

    // initialize configuration
    let cfg = &config::CONFIG;

    // define routes
    let routes = get().map(form::render_form).or(post()
        .and(body::content_length_limit(cfg.max_body_size()))
        .and(body::form())
        .and(path("dl"))
        .and_then(download::handle_download));

    // initialize logger
    init_logger(cfg);

    // print version and bind address
    log::info!("Running aa-fastlink {}", env!("CARGO_PKG_VERSION"));
    log::info!("Binding to http://{}:{}", cfg.bind_ip(), cfg.bind_port());

    // start server
    serve(routes).run((cfg.bind_ip(), cfg.bind_port())).await;
}

// initializes logger with custom settings
pub fn init_logger(cfg: &config::Config) {
    use std::io::Write;

    let mut builder = env_logger::Builder::new();

    builder.format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()));

    builder.filter_level(if cfg.debug_logging() {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    });

    builder.init();
}

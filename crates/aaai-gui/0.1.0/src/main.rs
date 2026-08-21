//! aaai-gui — GUI entry point (iced + snora).

mod app;
mod views;
mod style;
mod theme;

fn main() -> iced::Result {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn")
    ).init();

    iced::application(
        app::App::default,
        app::App::update,
        app::App::view,
    )
    .title(|_: &app::App| String::from("aaai — audit for asset integrity"))
    .run()
}

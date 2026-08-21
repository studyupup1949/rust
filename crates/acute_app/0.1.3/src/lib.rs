mod app;
mod builder;

pub use app::App;
pub use builder::AppBuilder;

pub trait State {
    // Called after engine init
    fn load(&mut self, _app: &App) {}
    // Called as fast as possible
    fn update(&mut self, _app: &App) {}
    // Called in a defined fixed interval
    fn update_fixed(&mut self, _app: &App) {}
    // draw UI
    fn draw_ui(&mut self, _app: &App) {}
}


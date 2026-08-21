extern crate aardvark;
extern crate log;

use aardvark::Application;
use log::info;

fn main() {
    let _app = Application::create();
    info!("Hello World");
}

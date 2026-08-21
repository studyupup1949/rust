extern crate log;

use crate::logger;
use log::debug;

pub struct Application {}

impl Application {
    pub fn create() -> Application {
        let app = Application {};
        logger::init();
        debug!("Application created.");

        app
    }
}

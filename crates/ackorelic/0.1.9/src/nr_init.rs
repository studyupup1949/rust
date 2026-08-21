use crate::App;
use std::env;
use std::str::FromStr;

lazy_static! {
    pub static ref NR_APP: App = {
        let license_key = env::var("NEW_RELIC_LICENSE_KEY").unwrap_or_else(|_| "".to_string());
        let app_name =
            env::var("NEW_RELIC_APP_NAME").unwrap_or_else(|_| "acko_api_test".to_string());
        let app = App::new(&app_name, &license_key).expect("Could not create app");
        app
    };
    pub static ref ENABLE_NEW_RELIC: bool = {
        let enable_nr = env::var("ENABLE_NEW_RELIC").unwrap_or_else(|_| "false".to_string());
        let x: bool = FromStr::from_str(&enable_nr).unwrap();
        x
    };
}

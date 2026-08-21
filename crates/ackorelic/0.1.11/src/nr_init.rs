use crate::App;
use std::env;
use std::panic;

lazy_static! {
    pub static ref NR_APP: App = {
        let license_key = env::var("NEW_RELIC_LICENSE_KEY").unwrap_or_else(|_| "".to_string());
        let app_name =
            env::var("NEW_RELIC_APP_NAME").unwrap_or_else(|_| "acko_api_test".to_string());
        let app = App::new(&app_name, &license_key).expect("Could not create app");
        app
    };
    pub static ref ENABLE_NEW_RELIC: bool = {
        if env::var("ENABLE_NEW_RELIC") == Ok("true".to_string()) {
            if panic::catch_unwind(|| NR_APP.status()).is_ok() {
                true
            } else {
                println!("error_in_new_relic_integration");
                false
            }
        } else {
            false
        }
    };
}

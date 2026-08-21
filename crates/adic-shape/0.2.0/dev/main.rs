//! Dev site for testing adic-shape components

#![cfg(feature="dev")]
#![cfg(feature="leptos")]
#![allow(unreachable_pub)]

mod app;

use leptos::prelude::*;
use app::App;

/// Dev site main
pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    mount_to_body(|| {
        view! { <App/> }
    });
}

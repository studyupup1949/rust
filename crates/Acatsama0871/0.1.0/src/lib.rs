//! # Acatsama0871
//!
//! The first package of acatsama 0871.

pub use self::greetings::hello;

pub mod greetings {
    /// Combines two primary colors in equal amounts to create
    /// a secondary color.
    pub fn hello() -> String {
        String::from("Hello World!")
    }
}

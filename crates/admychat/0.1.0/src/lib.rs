use zed_extension_api::{self as zed, Result};

struct AdMyChatExtension;

impl zed::Extension for AdMyChatExtension {
    fn new() -> Self {
        AdMyChatExtension
    }
}

zed::register_extension!(AdMyChatExtension);

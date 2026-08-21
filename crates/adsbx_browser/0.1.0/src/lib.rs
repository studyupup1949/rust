#![deny(missing_docs)]
//! Tools for taking screenshots of the ADS-B Exchange map.

use std::sync::Arc;
use std::thread;

use headless_chrome::{
    protocol::{page::ScreenshotFormat, target::methods::CreateTarget},
    Browser,
};
use url::Url;

mod error;
pub use crate::error::Error;

/// Type wrapping a headless Chrome browser for accessing ADS-B Exchange maps.
pub struct AdsbxBrowser {
    /// The headless Chrome browser.
    // We keep a reference to the browser so it doesn't get dropped.
    #[allow(dead_code)]
    pub browser: Browser,
    /// The browser tab.
    pub tab: Arc<headless_chrome::Tab>,
    initialized: bool,
    show_track_labels: bool,
    show_labels: bool,
}

impl AdsbxBrowser {
    /// Creates a new ADS-B Exchange browser with the given dimensions.
    pub fn new(dimensions: (i32, i32)) -> std::result::Result<AdsbxBrowser, Error> {
        let browser = Browser::default()?;
        let (width, height) = dimensions;
        let tab = browser.new_tab_with_options(CreateTarget {
            url: "chrome://version",
            width: Some(width),
            height: Some(height),
            browser_context_id: None,
            enable_begin_frame_control: None,
        })?;
        Ok(AdsbxBrowser {
            browser,
            tab,
            initialized: false,
            show_track_labels: true,
            show_labels: true,
        })
    }

    /// Switches track labels on or off.
    pub fn show_track_labels(mut self, enabled: bool) -> Self {
        self.show_track_labels = enabled;
        self
    }

    /// Siwtches labels on or off.
    pub fn show_labels(mut self, enabled: bool) -> Self {
        self.show_labels = enabled;
        self
    }

    /// Takes a screenshot of a URL. Returns a screenshot image in PNG format.
    pub fn screenshot(&mut self, url: &Url) -> Result<Screenshot, Error> {
        let long_delay = std::time::Duration::from_millis(4000);
        let short_delay = std::time::Duration::from_millis(100);
        self.tab.navigate_to(url.as_str())?;
        self.tab.wait_until_navigated()?;
        thread::sleep(long_delay);
        // Delete the infoblock.
        self.tab
            .wait_for_element("#selected_infoblock")?
            .call_js_fn("function() { this.remove(); }", false)?;
        thread::sleep(short_delay);
        // Only need to turn on these labels once per browser; their state is
        // stored by the site in cookies.
        if !self.initialized {
            if self.show_track_labels {
                // Show timestamps, altitudes and speeds along each track.
                self.tab.press_key("k")?;
                thread::sleep(short_delay);
            }
            if self.show_labels {
                // Show aircraft registration labels.
                self.tab.press_key("l")?;
                thread::sleep(short_delay);
            }
            self.initialized = true;
        }
        // Close the ad div.
        #[allow(unused_must_use)]
        if let Ok(close) = self.tab.wait_for_element("[Title=\"Close\"]") {
            close.click();
            thread::sleep(short_delay);
        }
        let viewport = self
            .tab
            .wait_for_element("#map_container")?
            .get_box_model()?
            .margin_viewport();
        let image_data =
            self.tab
                .capture_screenshot(ScreenshotFormat::PNG, Some(viewport), true)?;
        Ok(Screenshot { data: image_data })
    }
}

/// Holds screenshot data.
pub struct Screenshot {
    /// The binary image data.
    pub data: Vec<u8>,
}

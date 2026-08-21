#![deny(missing_docs)]
//! Tools for taking screenshots of the ADS-B Exchange map.
//!
//! ```
//! use adsbx_screenshot::{AdsbxBrowser, AdsbxBrowserOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = AdsbxBrowserOptions {
//!     regs: vec!["N822LA".to_string()],
//!     ..Default::default()
//! };
//! let mut browser = AdsbxBrowser::new((1920, 1080))?;
//! let screenshot = browser.screenshot(&config)?;
//! // screenshot.data is a Vec<u8> containing the PNG data
//!
//! # Ok(())
//! # }
//! ```

use chrono::prelude::*;
use headless_chrome::{
    protocol::{page::ScreenshotFormat, target::methods::CreateTarget},
    Browser,
};
use std::sync::Arc;
use std::thread;
use url::Url;

mod error;
pub use crate::error::Error;

/// Options for configuring the ADS-B Exchange front-end.
pub struct AdsbxBrowserOptions {
    /// The base URL.
    pub base_url: Url,
    /// Delete ads?
    pub delete_ads: bool,
    /// Show aircraft labels?
    pub show_labels: bool,
    /// Show aircraft track labels?
    pub show_track_labels: bool,
    /// Hide the aircraft info block?
    pub hide_infoblock: bool,
    /// Show all aircraft, not just the selected one?
    pub no_isolation: bool,
    /// Hide the buttons?
    pub hide_buttons: bool,
    /// Hide the sidebar?
    pub hide_sidebar: bool,
    /// Zoom level.
    pub zoom: f32,
    /// Registration numbers of aircraft of interest
    pub regs: Vec<String>,
    /// ICAOs of aircraft of interest
    pub icaos: Vec<String>,
    /// Coordinates of center of map.
    pub coords: Option<Coordinates>,
    /// History options.
    pub history: Option<HistoryOptions>,
}

impl Default for AdsbxBrowserOptions {
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://globe.adsbexchange.com").unwrap(),
            delete_ads: true,
            show_labels: true,
            show_track_labels: false,
            hide_infoblock: true,
            no_isolation: true,
            hide_buttons: true,
            hide_sidebar: true,
            zoom: 13.0,
            regs: vec![],
            icaos: vec![],
            coords: None,
            history: None,
        }
    }
}

impl AdsbxBrowserOptions {
    /// Converts a set of options into a URL.
    pub fn to_url(&self) -> Url {
        let mut url = self.base_url.clone();
        if self.no_isolation {
            url.query_pairs_mut().append_key_only("noIsolation");
        }
        if self.hide_buttons {
            url.query_pairs_mut().append_key_only("hideButtons");
        }
        if self.hide_sidebar {
            url.query_pairs_mut().append_key_only("hideSidebar");
        }
        url.query_pairs_mut()
            .append_pair("zoom", &format!("{:.1}", self.zoom));
        if !self.icaos.is_empty() {
            url.query_pairs_mut()
                .append_pair("icao", &self.icaos.join(","));
        }
        if !self.regs.is_empty() {
            url.query_pairs_mut()
                .append_pair("reg", &self.regs.join(","));
        }
        if let Some(coord) = &self.coords {
            url.query_pairs_mut()
                .append_pair("lat", &coord.lat.to_string())
                .append_pair("lon", &coord.lon.to_string());
        }
        if let Some(history) = &self.history {
            match history {
                HistoryOptions::EntireDay(date) => {
                    url.query_pairs_mut()
                        .append_pair("showTrace", &date.format("%Y-%m-%d").to_string());
                }
                HistoryOptions::TimeRange(start, end) => {
                    url.query_pairs_mut()
                        .append_pair("showTrace", &start.format("%Y-%m-%d").to_string())
                        .append_pair("startTime", &start.format("%H:%M:%S").to_string())
                        .append_pair("endTime", &end.format("%H:%M:%S").to_string());
                }
            }
        }
        url
    }
}

/// Lat/Lon pair.
pub struct Coordinates {
    /// Latitude.
    pub lat: f64,
    /// Longitude.
    pub lon: f64,
}

/// History options
#[derive(Debug, Clone)]
pub enum HistoryOptions {
    /// Show the history of this entire day.
    EntireDay(Date<Utc>),
    /// Show the history between these two times.
    TimeRange(DateTime<Utc>, DateTime<Utc>),
}

/// Type wrapping a headless Chrome browser for accessing ADS-B Exchange maps.
pub struct AdsbxBrowser {
    /// The headless Chrome browser.
    // We keep a reference to the browser so it doesn't get dropped.
    pub browser: Browser,
    /// The browser tab.
    pub tab: Arc<headless_chrome::Tab>,
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
            show_track_labels: false,
            show_labels: false,
        })
    }

    /// Takes a screenshot of a URL. Returns a screenshot image in PNG format.
    pub fn screenshot(&mut self, options: &AdsbxBrowserOptions) -> Result<Screenshot, Error> {
        let long_delay = std::time::Duration::from_millis(4000);
        let short_delay = std::time::Duration::from_millis(100);
        self.tab.navigate_to(options.to_url().as_str())?;
        self.tab.wait_until_navigated()?;
        thread::sleep(long_delay);
        let mut need_short_delay = false;
        if options.hide_infoblock {
            // Delete the infoblock.
            self.tab
                .wait_for_element("#selected_infoblock")?
                .call_js_fn("function() { this.remove(); }", false)?;
            need_short_delay = true;
        }
        // Only need to turn on these labels once per browser; their state is
        // stored by the site in cookies.
        if self.show_track_labels != options.show_track_labels {
            // Toggle timestamps, altitudes and speeds along each track.
            self.tab.press_key("k")?;
            need_short_delay = true;
        }
        if self.show_labels != options.show_track_labels {
            // Toggle aircraft registration labels.
            self.tab.press_key("l")?;
            need_short_delay = true;
        }
        // Close the ad div.
        #[allow(unused_must_use)]
        if let Ok(close) = self.tab.wait_for_element("[Title=\"Close\"]") {
            close.click();
            need_short_delay = true;
        }
        if need_short_delay {
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

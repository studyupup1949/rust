#![deny(missing_docs)]
//! Tools for taking screenshots of the ADS-B Exchange map.
//!
//! ```
//! use adsbx_screenshot::{AdsbxBrowser, AdsbxBrowserOptions, HistoryOptions};
//! use chrono::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = AdsbxBrowserOptions {
//!     regs: vec!["N822LA".to_string()],
//!     history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 3))),
//!     zoom: 13.0,
//!     delete_ads: true,
//!     show_track_labels: true,
//!     hide_infoblock: false,
//!     ..Default::default()
//! };
//!
//! let mut browser = AdsbxBrowser::new((800, 600))?;
//! let screenshot = browser.screenshot(&config)?;
//! // screenshot.data is a Vec<u8> containing the PNG data
//! let mut file = std::fs::File::create("screenshot.jpg").unwrap();
//! std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
//!
//! # Ok(())
//! # }
//! ```

use chrono::{format::StrftimeItems, prelude::*};
use headless_chrome::{
    protocol::{page::ScreenshotFormat, target::methods::CreateTarget},
    Browser,
};
use std::thread;
use std::{num::ParseFloatError, sync::Arc};
use url::Url;

mod error;
pub use crate::error::Error;

/// Latitude, longitude pair.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Coordinates {
    /// Latitude.
    pub lat: f64,
    /// Longitude.
    pub lon: f64,
}

/// History options.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum HistoryOptions {
    /// Show the history for this entire day.
    EntireDay(Date<Utc>),
    /// Show the history between these two times.
    TimeRange(DateTime<Utc>, DateTime<Utc>),
}

/// Options for configuring the ADS-B Exchange front-end.
///
/// Some of these options correspond to URL query parameters while others are
/// only selectable in the ADS-B Exchange web user interface, and some control
/// browser behavior (e.g. whether to delete ads).
///
/// You can parse an AdsbxBrowserOptions from a URL or URL string using
/// `try_from`:
///
/// ```
/// use adsbx_screenshot::AdsbxBrowserOptions;
///
/// let options = AdsbxBrowserOptions::try_from(
///     "https://globe.adsbexchange.com/?icao=ab5611,ab34a2,a4064c&lat=42.763&lon=-71.741&zoom=10.3&showTrace=2021-12-07&trackLabels"
/// ).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
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
    /// The map layer to use. See
    /// <https://github.com/wiedehopf/tar1090/blob/master/html/layers.js> for
    /// layer names.
    ///
    /// For example, to select the ESRI satellite base layer, specify a value of
    /// `"esri"`.
    pub layer: Option<String>,
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
    /// Other URL query parameters not handled by the other fields.
    pub other_params: Vec<(String, Option<String>)>,
}

impl Default for AdsbxBrowserOptions {
    /// Default options are optimized for screenshots, with ads deleted, showing
    /// labels, hiding the infoblock, buttons, and sidebar.  Default zoom level
    /// is 13.
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://globe.adsbexchange.com").unwrap(),
            delete_ads: true,
            show_labels: true,
            show_track_labels: false,
            hide_infoblock: true,
            layer: None,
            no_isolation: true,
            hide_buttons: true,
            hide_sidebar: true,
            zoom: 13.0,
            regs: vec![],
            icaos: vec![],
            coords: None,
            history: None,
            other_params: vec![],
        }
    }
}

impl AdsbxBrowserOptions {
    /// Converts a set of options into a URL.
    ///
    /// Note that not all options correspond to URL parameters, so converting to
    /// a URL is a lossy operation.
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
        for (key, value) in &self.other_params {
            if let Some(value) = value {
                url.query_pairs_mut().append_pair(key, value);
            } else {
                url.query_pairs_mut().append_key_only(key);
            }
        }
        url
    }
}

impl TryFrom<&str> for AdsbxBrowserOptions {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Url::parse(value) {
            Ok(url) => Self::try_from(&url),
            Err(err) => Err(Error::UrlStructureError(format!(
                "Could not parse url {}: {}",
                value,
                err.to_string()
            ))),
        }
    }
}

impl TryFrom<&Url> for AdsbxBrowserOptions {
    type Error = Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        let mut options = Self::default();
        let mut coords: (Option<f64>, Option<f64>) = (None, None);
        let mut date = None;
        let mut start_time = None;
        let mut end_time = None;
        for (key, value) in value.query_pairs() {
            match key.as_ref() {
                "icao" => options.icaos = value.split(',').map(|s| s.to_string()).collect(),
                "reg" => options.regs = value.split(',').map(|s| s.to_string()).collect(),
                "lat" => {
                    coords.0 = Some(value.parse().map_err(|e: ParseFloatError| {
                        Error::UrlStructureError(format!(
                            "Could not parse lat '{}': {}",
                            value,
                            e.to_string()
                        ))
                    })?)
                }
                "lon" => {
                    coords.1 = Some(value.parse().map_err(|e: ParseFloatError| {
                        Error::UrlStructureError(format!(
                            "Could not parse lon '{}': {}",
                            value,
                            e.to_string()
                        ))
                    })?)
                }
                "showTrace" => date = Some(value.to_string()),
                "startTime" => start_time = Some(value.to_string()),
                "endTime" => end_time = Some(value.to_string()),
                // "showTrace" => {
                // options.history = Some(HistoryOptions::EntireDay(Date::<Utc>::from_utc(
                //     NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|e| {
                //         Error::UrlStructureError(format!(
                //             "Could not parse showTrace '{}': {}",
                //             value,
                //             e.to_string()
                //         ))
                //     })?,
                //     Utc,
                // )));
                // }
                // "startTime" => {
                //     start_end_times.0 = Some(DateTime::<Utc>::from_utc(
                //         NaiveDateTime::parse_from_str(&value, "%H:%M:%S").map_err(|e| {
                //             Error::UrlStructureError(format!(
                //                 "Could not parse startTime '{}': {}",
                //                 value,
                //                 e.to_string()
                //             ))
                //         })?,
                //         Utc,
                //     ));
                // }
                // "endTime" => {
                //     start_end_times.1 = Some(DateTime::<Utc>::from_utc(
                //         NaiveDateTime::parse_from_str(&value, "%H:%M:%S").map_err(|e| {
                //             Error::UrlStructureError(format!(
                //                 "Could not parse endTime '{}': {}",
                //                 value,
                //                 e.to_string()
                //             ))
                //         })?,
                //         Utc,
                //     ));
                // }
                "noIsolation" => options.no_isolation = true,
                "hideButtons" => options.hide_buttons = true,
                "hideSidebar" => options.hide_sidebar = true,
                "zoom" => {
                    options.zoom = value.parse().map_err(|e: ParseFloatError| {
                        Error::UrlStructureError(format!(
                            "Could not parse zoom '{}': {}",
                            value,
                            e.to_string()
                        ))
                    })?
                }
                _ => options
                    .other_params
                    .push((key.to_string(), Some(value.to_string()))),
            }
        }
        // Check for highler level consistency, e.g. if you specify lat you must
        // also specify lon.
        match coords {
            (None, None) => {}
            (None, Some(_)) => {
                return Err(Error::UrlStructureError(
                    "lat= is missing but lon= is present".to_string(),
                ))
            }
            (Some(_), None) => {
                return Err(Error::UrlStructureError(
                    "lon= is missing but lat= is present".to_string(),
                ))
            }
            (Some(lat), Some(lon)) => options.coords = Some(Coordinates { lat, lon }),
        }
        match (date, start_time, end_time) {
            (None, None, None) => {}
            (None, None, Some(_)) => {
                return Err(Error::UrlStructureError(
                    "endTime is present but showTrace and startTime are missing".to_string(),
                ))
            }
            (None, Some(_), None) => {
                return Err(Error::UrlStructureError(
                    "startTime is present but showTrace and endTime are missing".to_string(),
                ))
            }
            (None, Some(_), Some(_)) => {
                return Err(Error::UrlStructureError("showTrace is missing".to_string()))
            }
            (Some(date), None, None) => {
                options.history = Some(HistoryOptions::EntireDay(Date::<Utc>::from_utc(
                    NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| {
                        Error::UrlStructureError(format!(
                            "Could not parse showTrace '{}': {}",
                            date,
                            e.to_string()
                        ))
                    })?,
                    Utc,
                )))
            }
            (Some(_), None, Some(_)) => {
                return Err(Error::UrlStructureError(
                    "showTrace and endTime are present but startTime is missing".to_string(),
                ))
            }
            (Some(_), Some(_), None) => {
                return Err(Error::UrlStructureError(
                    "showTrace and startTime are present but endTime is missing".to_string(),
                ))
            }
            (Some(date), Some(start_time), Some(end_time)) => {
                options.history =
                    Some(HistoryOptions::TimeRange(
                        parse_date_with_time(&date, &start_time).map_err(|e| {
                            Error::UrlStructureError(format!(
                            "Could not parse showTrace '{}' and startTime '{}' as a DateTime: {}",
                            date, start_time, e.to_string()
                        ))
                        })?,
                        parse_date_with_time(&date, &end_time).map_err(|e| {
                            Error::UrlStructureError(format!(
                            "Could not parse showTrace '{}' and startTime '{}' as a DateTime: {}",
                            date, end_time, e.to_string()
                        ))
                        })?,
                    ));
            }
        }
        Ok(options)
    }
}

fn parse_date_with_time(date: &str, time: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let mut p = chrono::format::Parsed::default();
    chrono::format::parse(&mut p, date, StrftimeItems::new("%Y-%m-%d"))?;
    chrono::format::parse(&mut p, time, StrftimeItems::new("%H:%M"))?;
    p.to_datetime_with_timezone(&Utc)
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
        thread::sleep(short_delay);
        let mut need_short_delay = false;
        if let Some(layer) = &options.layer {
            let mut script = String::new();
            script.push_str("ol.control.LayerSwitcher.forEachRecursive(layers_group, l => { if (l.get('name') == '");
            script.push_str(layer);
            script.push_str("') { console.log(l.get('name')); ol.control.LayerSwitcher.setVisible_(OLMap, l, true); } });");
            self.tab.evaluate(script.as_str(), false)?;
        }
        thread::sleep(long_delay);
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
            self.show_track_labels = options.show_track_labels;
            need_short_delay = true;
        }
        if self.show_labels != options.show_labels {
            // Toggle aircraft registration labels.
            self.tab.press_key("l")?;
            self.show_labels = options.show_labels;
            need_short_delay = true;
        }
        if options.delete_ads {
            // Close the ad div.
            #[allow(unused_must_use)]
            if let Ok(close) = self.tab.wait_for_element("[Title=\"Close\"]") {
                close.click();
                need_short_delay = true;
            }
        }
        if need_short_delay {
            thread::sleep(short_delay);
        }
        let viewport = self
            .tab
            .wait_for_element("#map_container")?
            .get_box_model()?
            .margin_viewport();
        if viewport.width == 72.0 {
            return Err(Error::ChromeError(
                "Viewport bug: width is too small".to_string(),
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    // use pretty_assertions::assert_eq;

    #[test]
    fn test_browser_options_to_url() {
        let options = AdsbxBrowserOptions {
            zoom: 1.0,
            ..AdsbxBrowserOptions::default()
        };
        assert_eq!(
            options.to_url().as_str(),
            "https://globe.adsbexchange.com/?noIsolation&hideButtons&hideSidebar&zoom=1.0"
        );
        let options = AdsbxBrowserOptions {
            no_isolation: false,
            hide_buttons: false,
            hide_sidebar: false,
            ..AdsbxBrowserOptions::default()
        };
        assert_eq!(
            options.to_url().as_str(),
            "https://globe.adsbexchange.com/?zoom=13.0"
        );
        let options = AdsbxBrowserOptions {
            regs: vec!["ABC123".to_string(), "DEF456".to_string()],
            icaos: vec!["000000".to_string(), "AE0001".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 7))),
            other_params: vec![
                ("foo".to_string(), None),
                ("bar".to_string(), Some("baz".to_string())),
            ],
            ..AdsbxBrowserOptions::default()
        };
        assert_eq!(
            options.to_url().as_str(),
            "https://globe.adsbexchange.com/?noIsolation&hideButtons&hideSidebar&zoom=13.0&icao=000000%2CAE0001&reg=ABC123%2CDEF456&showTrace=2021-12-07&foo&bar=baz"
        );
    }

    #[test]
    fn test_browser_options_from_url() {
        assert_eq!(
            AdsbxBrowserOptions::try_from(
                &Url::parse(
                    "https://globe.adsbexchange.com/?noIsolation&hideButtons&hideSidebar&zoom=1.0"
                )
                .unwrap()
            )
            .unwrap(),
            AdsbxBrowserOptions {
                zoom: 1.0,
                no_isolation: true,
                ..AdsbxBrowserOptions::default()
            }
        );
        assert_eq!(
            AdsbxBrowserOptions::try_from(
                &Url::parse("https://globe.adsbexchange.com/?noIsolation&hideButtons&hideSidebar&zoom=13.0&icao=000000%2CAE0001&reg=ABC123%2CDEF456&showTrace=2021-12-07&foo=&bar=baz"
            )
            .unwrap()
        )
        .unwrap(),
        AdsbxBrowserOptions {
            zoom: 13.0,
            regs: vec!["ABC123".to_string(), "DEF456".to_string()],
            icaos: vec!["000000".to_string(), "AE0001".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 7))),
            other_params: vec![
                ("foo".to_string(), Some("".to_string())),
                ("bar".to_string(), Some("baz".to_string())),
            ],
            ..AdsbxBrowserOptions::default()
        });
        assert_eq!(
            AdsbxBrowserOptions::try_from(
                "https://globe.adsbexchange.com/?noIsolation&hideButtons&hideSidebar&zoom=13.0&icao=000000%2CAE0001&reg=ABC123%2CDEF456&showTrace=2021-12-07&foo=&bar=baz"
        )
        .unwrap(),
        AdsbxBrowserOptions {
            zoom: 13.0,
            regs: vec!["ABC123".to_string(), "DEF456".to_string()],
            icaos: vec!["000000".to_string(), "AE0001".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 7))),
            other_params: vec![
                ("foo".to_string(), Some("".to_string())),
                ("bar".to_string(), Some("baz".to_string())),
            ],
            ..AdsbxBrowserOptions::default()
        });
        assert_eq!(
            AdsbxBrowserOptions::try_from(
                "https://globe.adsbexchange.com/?noIsolation&hideButtons&hideSidebar&zoom=13.0&icao=AE0001&showTrace=2021-12-07&startTime=12:51&endTime=12:58"
        )
        .unwrap(),
        AdsbxBrowserOptions {
            zoom: 13.0,
            icaos: vec!["AE0001".to_string()],
            history: Some(HistoryOptions::TimeRange(Utc.ymd(2021, 12, 7).and_hms(12, 51, 0), Utc.ymd(2021, 12, 7).and_hms(12, 58, 0))),
            ..AdsbxBrowserOptions::default()
        });
    }
}

#[cfg(test)]
mod doctests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn doctest() {
        let config = AdsbxBrowserOptions {
            regs: vec!["N822LA".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 3))),
            zoom: 13.0,
            delete_ads: true,
            show_track_labels: true,
            hide_infoblock: false,
            ..Default::default()
        };
        let mut browser = AdsbxBrowser::new((1920, 1080)).unwrap();
        let screenshot = browser.screenshot(&config).unwrap();
        let mut file = std::fs::File::create("doctest.jpg").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
    }
}

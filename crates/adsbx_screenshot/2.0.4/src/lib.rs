#![deny(missing_docs)]
//! Tools for taking screenshots of the ADS-B Exchange map.
//!
//! ```
//! use adsbx_screenshot::{AdsbxBrowser, AdsbxBrowserOptions, HistoryOptions, KmlType};
//! use chrono::prelude::*;
//! use std::path::PathBuf;
//! use tempfile::tempdir;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a temporary directory for downloads in the example
//! let temp_dir = tempdir()?;
//! let download_dir = temp_dir.path().to_path_buf();
//!
//! let config = AdsbxBrowserOptions {
//!     regs: vec!["N822LA".to_string()],
//!     // Use NaiveDate for EntireDay
//!     history: Some(HistoryOptions::EntireDay(Utc.with_ymd_and_hms(2021, 12, 3, 0, 0, 0).unwrap().date_naive())),
//!     zoom: 13.0,
//!     delete_ads: true,
//!     show_track_labels: true,
//!     hide_infoblock: false,
//!     request_screenshot: true, // Explicitly request screenshot
//!     request_kml: Some(KmlType::Baro), // Request KML with Barometric altitude
//!     download_dir: download_dir.clone(), // Specify download directory for KML
//!     ..Default::default()
//! };
//!
//! let mut browser = AdsbxBrowser::new((1920, 1080))?;
//! let output = browser.get_output(&config)?;
//!
//! // Save the screenshot if it was generated
//! if let Some(screenshot) = output.screenshot {
//!     let screenshot_path = download_dir.join("screenshot.png");
//!     let mut file = std::fs::File::create(&screenshot_path)?;
//!     std::io::Write::write_all(&mut file, &screenshot.data)?;
//!     println!("Saved {}", screenshot_path.display());
//! }
//!
//! // Save the KML data if it was generated
//! if let Some(kml) = output.kml {
//!     let filename = format!("track_{}", kml.filename); // Prepend prefix
//!     let kml_path = download_dir.join(&filename);
//!     // File is already read by get_output, re-save for demonstration
//!     let mut file = std::fs::File::create(&kml_path)?;
//!     std::io::Write::write_all(&mut file, kml.data.as_bytes())?;
//!     println!("Saved {}", kml_path.display());
//!     // Clean up the re-saved KML file
//!     std::fs::remove_file(&kml_path)?;
//! }
//!
//! # Ok(())
//! # // temp_dir goes out of scope and is automatically cleaned up
//! # }
//! ```

use chrono::{format::StrftimeItems, prelude::*};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::Browser;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use std::{num::ParseFloatError, sync::Arc};
use url::Url;

// Imports needed for the new download logic
use headless_chrome::protocol::cdp::types::Method;
use serde::Serialize;
use tempfile::tempdir;

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
    EntireDay(NaiveDate), // Use NaiveDate instead of deprecated Date<Utc>
    /// Show the history between these two times.
    TimeRange(DateTime<Utc>, DateTime<Utc>),
}

/// The type of KML altitude data to request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KmlType {
    /// Barometric altitude.
    Baro,
    /// Geometric (GPS) altitude.
    Geom,
    /// Average of Barometric and Geometric altitude.
    Avg,
}

/// Holds downloaded KML data.
#[derive(Debug, Clone, PartialEq)]
pub struct KmlData {
    /// The filename suggested by the browser.
    pub filename: String,
    /// The KML content as a string.
    pub data: String,
}

/// Holds the requested output(s) from the browser interaction.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdsbxOutput {
    /// The screenshot data, if requested.
    pub screenshot: Option<Screenshot>,
    /// The KML data, if requested.
    pub kml: Option<KmlData>,
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
    /// Request a screenshot? Defaults to true.
    pub request_screenshot: bool,
    /// Request KML data? Defaults to None.
    pub request_kml: Option<KmlType>,
    /// Directory to download KML files to. Defaults to the current directory.
    pub download_dir: PathBuf,
}

impl Default for AdsbxBrowserOptions {
    /// Default options are optimized for screenshots, with ads deleted, showing
    /// labels, hiding the infoblock, buttons, and sidebar. Default zoom level
    /// is 13. KML is not requested by default.
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
            request_screenshot: true,
            request_kml: None,
            download_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
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
        if !self.icaos.is_empty() {
            println!("ICAOs: {:?}", self.icaos);
            url.query_pairs_mut()
                .append_pair("icao", &self.icaos.join(","));
        }
        println!("ICAOs: {:?}", self.icaos);
        // if self.no_isolation {
        //     url.query_pairs_mut().append_key_only("noIsolation");
        // }
        if self.hide_buttons {
            url.query_pairs_mut().append_key_only("hideButtons");
        }
        if self.hide_sidebar {
            url.query_pairs_mut().append_key_only("hideSidebar");
        }
        url.query_pairs_mut()
            .append_pair("zoom", &format!("{:.1}", self.zoom));
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
                value, err
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
                        Error::UrlStructureError(format!("Could not parse lat '{}': {}", value, e))
                    })?)
                }
                "lon" => {
                    coords.1 = Some(value.parse().map_err(|e: ParseFloatError| {
                        Error::UrlStructureError(format!("Could not parse lon '{}': {}", value, e))
                    })?)
                }
                "showTrace" => date = Some(value.to_string()),
                "startTime" => start_time = Some(value.to_string()),
                "endTime" => end_time = Some(value.to_string()),
                "noIsolation" => options.no_isolation = true,
                "hideButtons" => options.hide_buttons = true,
                "hideSidebar" => options.hide_sidebar = true,
                "zoom" => {
                    options.zoom = value.parse().map_err(|e: ParseFloatError| {
                        Error::UrlStructureError(format!("Could not parse zoom '{}': {}", value, e))
                    })?
                }
                _ => options
                    .other_params
                    .push((key.to_string(), Some(value.to_string()))),
            }
        }
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
                options.history = Some(HistoryOptions::EntireDay(
                    NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| {
                        Error::UrlStructureError(format!(
                            "Could not parse showTrace '{}': {}",
                            date, e
                        ))
                    })?,
                ))
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
                options.history = Some(HistoryOptions::TimeRange(
                    parse_date_with_time(&date, &start_time).map_err(|e| {
                        Error::UrlStructureError(format!(
                            "Could not parse showTrace '{}' and startTime '{}' as a DateTime: {}",
                            date, start_time, e
                        ))
                    })?,
                    parse_date_with_time(&date, &end_time).map_err(|e| {
                        Error::UrlStructureError(format!(
                            "Could not parse showTrace '{}' and endTime '{}' as a DateTime: {}",
                            date, end_time, e
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

    // Try parsing with seconds first
    let time_fmt_ss = StrftimeItems::new("%H:%M:%S");
    if chrono::format::parse(&mut p, time, time_fmt_ss).is_err() {
        // If parsing with seconds failed, try parsing without seconds
        let time_fmt = StrftimeItems::new("%H:%M");
        // Reset seconds/subseconds potentially partially parsed by the failed attempt
        p.second = None;
        p.nanosecond = None;
        chrono::format::parse(&mut p, time, time_fmt)?; // Return error if this also fails
    }

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
    pub fn new(dimensions: (u32, u32)) -> std::result::Result<AdsbxBrowser, Error> {
        let browser_options = headless_chrome::LaunchOptionsBuilder::default()
            .sandbox(false)
            .headless(true)
            .build()
            .map_err(|e| Error::BrowserError(format!("Could not create browser options: {}", e)))?;
        let browser = Browser::new(browser_options)
            .map_err(|e| Error::BrowserError(format!("Could not create browser: {}", e)))?;
        let (width, height) = dimensions;
        let tab = browser
            .new_tab_with_options(CreateTarget {
                url: "chrome://version".to_string(),
                width: Some(width),
                height: Some(height),
                browser_context_id: None,
                enable_begin_frame_control: None,
                new_window: Some(true),
                background: None,
                for_tab: None, // Add missing field
            })
            .map_err(|e| Error::BrowserError(format!("Could not create tab: {}", e)))?;
        Ok(AdsbxBrowser {
            browser,
            tab,
            show_track_labels: false,
            show_labels: false,
        })
    }

    /// Prepares the browser page by navigating and applying UI options.
    fn prepare_page(&mut self, options: &AdsbxBrowserOptions) -> Result<(), Error> {
        let long_delay = std::time::Duration::from_millis(4000);
        let short_delay = std::time::Duration::from_millis(300);
        println!("Navigating to {}", options.to_url());
        self.tab
            .navigate_to(options.to_url().as_str())
            .map_err(|e| {
                Error::BrowserError(format!("Could not navigate to {}: {}", options.to_url(), e))
            })?;
        self.tab.wait_until_navigated().map_err(|e| {
            Error::BrowserError(format!(
                "Could not wait for navigation to {}: {}",
                options.to_url(),
                e
            ))
        })?;
        // Small delay for initial elements to settle?
        thread::sleep(short_delay);
        // Set layer if specified
        if let Some(layer) = &options.layer {
            let mut script = String::new();
            script.push_str(
                "ol.control.LayerSwitcher.forEachRecursive(layers_group, l => { if (l.get('name') == '",
            );
            script.push_str(layer);
            script.push_str("') { console.log(l.get('name')); ol.control.LayerSwitcher.setVisible_(OLMap, l, true); } });");
            self.tab.evaluate(script.as_str(), false).map_err(|e| {
                Error::BrowserError(format!("Could not set layer to {}: {}", layer, e))
            })?;
            // Layer switching might need a longer delay
            thread::sleep(long_delay);
        } else {
            // Even without layer switching, wait for map tiles etc.
            thread::sleep(long_delay);
        }

        Ok(())
    }

    /// Takes a screenshot of the current view.
    pub fn take_screenshot(&mut self, options: &AdsbxBrowserOptions) -> Result<Screenshot, Error> {
        let short_delay = std::time::Duration::from_millis(300);
        let mut ui_update_needed = false;
        if options.hide_infoblock {
            // Try to delete the infoblock.
            match self.tab.wait_for_element("#selected_infoblock") {
                Ok(elem) => {
                    elem.call_js_fn("function() { this.remove(); }", vec![], false)
                        .map_err(|e| {
                            Error::BrowserError(format!("Could not remove infoblock: {}", e))
                        })?;
                    ui_update_needed = true;
                }
                Err(_e) => {
                    // It might not exist if no plane is selected, which is fine
                }
            }
        }
        // if options.hide_sidebar {
        //     // Press 'S' to toggle sidebar hidden
        //     self.tab.press_key("S").map_err(|e| {
        //         Error::BrowserError(format!("Could not press 'S' to hide sidebar: {}", e))
        //     })?;
        //     ui_update_needed = true;
        // }
        // Apply label visibility based on current state vs options
        // Only need to turn on these labels once per browser; their state is
        // stored by the site in cookies.
        if self.show_track_labels != options.show_track_labels {
            // Toggle timestamps, altitudes and speeds along each track.
            self.tab.press_key("k").map_err(|e| {
                Error::BrowserError(format!("Could not toggle track labels: {}", e))
            })?;
            self.show_track_labels = options.show_track_labels;
            ui_update_needed = true; // Assume label toggles might need delay too
        }
        if self.show_labels != options.show_labels {
            // Toggle aircraft registration labels.
            self.tab
                .press_key("l")
                .map_err(|e| Error::BrowserError(format!("Could not toggle labels: {}", e)))?;
            self.show_labels = options.show_labels;
            ui_update_needed = true; // Assume label toggles might need delay too
        }
        // If any UI updates happened, wait a bit for them to apply
        if ui_update_needed {
            thread::sleep(short_delay);
        }
        let viewport = self
            .tab
            .wait_for_element("#map_container")
            .map_err(|e| Error::BrowserError(format!("Could not wait for map: {}", e)))?
            .get_box_model()
            .map_err(|e| Error::BrowserError(format!("Could not get map box model: {}", e)))?
            .margin_viewport();
        let image_data = self
            .tab
            .capture_screenshot(
                CaptureScreenshotFormatOption::Png,
                None,
                Some(viewport),
                true, // from_surface
            )
            .map_err(|e| Error::BrowserError(format!("Could not capture screenshot: {}", e)))?;
        Ok(Screenshot { data: image_data })
    }

    /// Downloads the KML file for the specified altitude type.
    fn download_kml(
        &self,
        options: &AdsbxBrowserOptions,
        kml_type: KmlType,
    ) -> Result<KmlData, Error> {
        // Create a temporary directory for the download
        let temp_dir = tempdir().map_err(|e| Error::BrowserError(format!("Could not create temporary directory: {}", e)))?;
        let temp_dir_path = temp_dir.path();
        let temp_dir_path_str = temp_dir_path.to_str().ok_or_else(|| {
            Error::BrowserError(format!(
                "Temporary directory path {:?} is not valid UTF-8",
                temp_dir_path
            ))
        })?;
        // Define the command to set download behavior to the temporary directory
        #[derive(Serialize, Debug)]
        struct SetDownloadBehaviorCommand<'a> {
            behavior: &'static str,
            #[serde(rename = "downloadPath")]
            download_path: &'a str,
        }
        impl Method for SetDownloadBehaviorCommand<'_> {
            const NAME: &'static str = "Page.setDownloadBehavior";
            type ReturnObject = serde_json::Value; // Or define a specific struct if needed
        }
        // Send the command to allow downloads to the temporary directory
        let command = SetDownloadBehaviorCommand {
            behavior: "allow",
            download_path: temp_dir_path_str,
        };
        self.tab
            .call_method(command)
            .map_err(|e| Error::BrowserError(format!("Could not set download behavior: {}", e)))?;
        let button_id = match kml_type {
            KmlType::Baro => "#export_kml_baro",
            KmlType::Geom => "#export_kml_geom",
            KmlType::Avg => "#export_kml_geom_avg",
        };
        // Find and click the download button
        let button = self.tab.wait_for_element(button_id).map_err(|e| {
            Error::BrowserError(format!("Could not find KML button {}: {}", button_id, e))
        })?;
        button.click().map_err(|e| {
            Error::BrowserError(format!("Could not click KML button {}: {}", button_id, e))
        })?;

        // Monitor the temporary download directory for the KML file
        let start_time = Instant::now();
        let timeout = Duration::from_secs(30); // Adjust timeout as needed
        let poll_interval = Duration::from_millis(200);
        loop {
            if start_time.elapsed() > timeout {
                // Temp dir is cleaned up automatically when temp_dir goes out of scope
                return Err(Error::BrowserError(format!(
                    "Timeout waiting for KML download in temporary directory {:?}",
                    temp_dir_path
                )));
            }

            let mut downloaded_file: Option<(PathBuf, String)> = None;
            // Read entries from the temporary directory
            let entries = fs::read_dir(temp_dir_path).map_err(|e| {
                Error::BrowserError(format!(
                    "Could not read temporary download dir {:?}: {}",
                    temp_dir_path, e
                ))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    Error::BrowserError(format!("Error reading directory entry: {}", e))
                })?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "kml") {
                    // Found a KML file in the temporary directory
                    if downloaded_file.is_some() {
                        // Multiple KML files found - this is unexpected.
                        // Temp dir (including all files) will be cleaned up automatically.
                        return Err(Error::BrowserError(format!(
                            "Multiple KML files found in temporary download directory {:?}",
                            temp_dir_path
                        )));
                    }
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    downloaded_file = Some((path, filename));
                    // Don't break here, continue checking for multiple files in this poll cycle
                }
            }

            if let Some((temp_download_path, filename)) = downloaded_file {
                // Read the downloaded file content from the temporary location
                // Add a small delay to ensure the file is fully written
                thread::sleep(Duration::from_millis(1000));
                let kml_content = fs::read_to_string(&temp_download_path).map_err(|e| {
                    // Temp dir is cleaned up automatically
                    Error::BrowserError(format!(
                        "Could not read downloaded KML file {:?} from temp dir: {}",
                        temp_download_path, e
                    ))
                })?;

                // Prepare the final destination
                let target_dir = &options.download_dir;
                fs::create_dir_all(target_dir).map_err(|e| Error::BrowserError(format!(
                    "Could not create target download directory {:?}: {}",
                    target_dir, e
                )))?;
                let target_path = target_dir.join(&filename);

                // Copy the file from the temporary directory to the final destination
                fs::copy(&temp_download_path, &target_path).map_err(|e| {
                     // Temp dir is cleaned up automatically
                    Error::BrowserError(format!(
                        "Could not copy KML file from {:?} to {:?}: {}",
                        temp_download_path, target_path, e
                    ))
                })?;

                // KML data read and file copied successfully.
                // The temp_dir (containing the original download) will be cleaned up automatically.
                return Ok(KmlData {
                    filename,
                    data: kml_content,
                });
            }

            thread::sleep(poll_interval);
        }
    }

    /// Navigates to the ADS-B Exchange map and retrieves the requested outputs (screenshot and/or KML).
    pub fn get_output(&mut self, options: &AdsbxBrowserOptions) -> Result<AdsbxOutput, Error> {
        if !options.request_screenshot && options.request_kml.is_none() {
            return Err(Error::ConfigurationError(
                "No output requested (screenshot or KML)".to_string(),
            ));
        }
        // Prepare the page: applies all options including hiding elements.
        self.prepare_page(options)?;
        let mut output = AdsbxOutput::default();
        // Attempt KML Download (will likely fail if sidebar/infoblock were hidden by prepare_page)
        if let Some(kml_type) = options.request_kml {
            match self.download_kml(options, kml_type) {
                Ok(kml_data) => output.kml = Some(kml_data),
                Err(e) => {
                    return Err(Error::BrowserError(format!("KML download failed: {}", e)));
                }
            }
        }

        // Attempt Screenshot (should reflect state after prepare_page)
        if options.request_screenshot {
            println!("Taking screenshot...");
            // Add a small final delay before screenshot just in case
            thread::sleep(std::time::Duration::from_millis(100));
            output.screenshot = Some(self.take_screenshot(options)?);
        }

        Ok(output)
    }
}

/// Holds screenshot data.
#[derive(Debug, Clone, PartialEq)]
pub struct Screenshot {
    /// The binary image data.
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused import
    // use tempfile::tempdir; // Use tempfile for doctest
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
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 7, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
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
            history: Some(HistoryOptions::EntireDay(
                NaiveDate::from_ymd_opt(2021, 12, 7).unwrap()
            )),
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
            history: Some(HistoryOptions::EntireDay(
                NaiveDate::from_ymd_opt(2021, 12, 7).unwrap()
            )),
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
            history: Some(HistoryOptions::TimeRange(
                Utc.with_ymd_and_hms(2021, 12, 7, 12, 51, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 12, 7, 12, 58, 0).unwrap()
            )),
            ..AdsbxBrowserOptions::default()
        });
    }

    #[test]
    fn test_browser_options_from_url_time_formats() {
        // Test HH:MM format
        assert_eq!(
            AdsbxBrowserOptions::try_from(
                "https://globe.adsbexchange.com/?showTrace=2021-12-07&startTime=12:51&endTime=12:58"
            )
            .unwrap()
            .history,
            Some(HistoryOptions::TimeRange(
                Utc.with_ymd_and_hms(2021, 12, 7, 12, 51, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 12, 7, 12, 58, 0).unwrap()
            ))
        );

        // Test HH:MM:SS format
        assert_eq!(
            AdsbxBrowserOptions::try_from(
                "https://globe.adsbexchange.com/?showTrace=2021-12-07&startTime=12:51:30&endTime=12:58:45"
            )
            .unwrap()
            .history,
            Some(HistoryOptions::TimeRange(
                Utc.with_ymd_and_hms(2021, 12, 7, 12, 51, 30).unwrap(),
                Utc.with_ymd_and_hms(2021, 12, 7, 12, 58, 45).unwrap()
            ))
        );

        // Test invalid format
        assert!(AdsbxBrowserOptions::try_from(
            "https://globe.adsbexchange.com/?showTrace=2021-12-07&startTime=12:51:30:00&endTime=12:58"
         ).is_err());
        assert!(AdsbxBrowserOptions::try_from(
            "https://globe.adsbexchange.com/?showTrace=2021-12-07&startTime=12:51&endTime=12:58:45:00"
         ).is_err());
        assert!(AdsbxBrowserOptions::try_from(
            "https://globe.adsbexchange.com/?showTrace=2021-12-07&startTime=bad&endTime=12:58"
        )
        .is_err());
    }
}

#[cfg(test)]
mod doctests {
    use super::*;
    // Removed unused import
    // use std::path::PathBuf;
    use tempfile::tempdir; // Use tempfile here as well

    #[test]
    fn doctest() {
        // Use a temporary directory for downloads
        let temp_dir = tempdir().expect("Failed to create temporary directory for doctest");
        let download_dir = temp_dir.path().to_path_buf();

        let config = AdsbxBrowserOptions {
            regs: vec!["N822LA".to_string()],
            history: Some(HistoryOptions::EntireDay(
                NaiveDate::from_ymd_opt(2021, 12, 3).unwrap(), // Use NaiveDate
            )),
            zoom: 13.0,
            delete_ads: true,
            show_track_labels: true,
            hide_infoblock: false,
            request_screenshot: true,           // Request screenshot
            request_kml: Some(KmlType::Baro),   // Request KML
            download_dir: download_dir.clone(), // Use temp dir path
            ..Default::default()
        };
        let mut browser = AdsbxBrowser::new((1920, 1080)).unwrap();
        let output = browser.get_output(&config).unwrap(); // Use get_output

        // Save the screenshot if it was generated
        if let Some(screenshot) = output.screenshot {
            let screenshot_path = download_dir.join("doctest_screenshot.png");
            let mut file = std::fs::File::create(&screenshot_path).unwrap();
            std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
            println!("Saved screenshot to {:?}", screenshot_path);
            assert!(
                screenshot.data.len() > 1000,
                "Screenshot data seems too small"
            );
        }

        // Save the KML data if it was generated
        if let Some(kml) = output.kml {
            let kml_path = download_dir.join(&kml.filename); // Path inside temp dir
                                                             // File should already be read and deleted by download_kml,
                                                             // but we save it again here for the test assertion.
            let mut file = std::fs::File::create(&kml_path).unwrap();
            std::io::Write::write_all(&mut file, kml.data.as_bytes()).unwrap();
            println!("Re-saved KML to {:?} for assertion", kml_path);
            assert!(kml.data.contains("<kml"), "KML data missing <kml> tag");
            assert!(
                kml.data.contains(&config.regs[0]),
                "KML data missing registration"
            ); // Check if reg is in KML name/content
            assert!(kml_path.exists(), "KML file was not re-saved correctly"); // Check it exists after re-saving
            let _ = std::fs::remove_file(&kml_path); // Clean up the re-saved file
        }

        // temp_dir handle goes out of scope here and cleans up the directory
    }
}

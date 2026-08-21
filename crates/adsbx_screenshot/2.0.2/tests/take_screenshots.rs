#[cfg(test)]
mod tests {
    use adsbx_screenshot::{AdsbxBrowser, AdsbxBrowserOptions, HistoryOptions, KmlType};
    use chrono::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn test_reg_now() {
        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            zoom: 10.0,
            ..AdsbxBrowserOptions::default()
        };
        let output = AdsbxBrowser::new((800, 600))
            .unwrap()
            .get_output(&options)
            .unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-maybe-n737dw-now.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
    }

    #[test]
    fn test_reg_history() {
        let mut browser = AdsbxBrowser::new((800, 600)).unwrap();
        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::TimeRange(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 12, 9, 1, 0, 0).unwrap(),
            )),
            zoom: 8.0,
            ..AdsbxBrowserOptions::default()
        };
        let output = browser.get_output(&options).unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-n737dw-1-hour.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
            zoom: 8.0,
            ..AdsbxBrowserOptions::default()
        };
        let output = browser.get_output(&options).unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-n737dw-all-day.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
            zoom: 9.0,
            show_track_labels: true,
            ..AdsbxBrowserOptions::default()
        };
        let output = browser.get_output(&options).unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-n737dw-track-labels.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
            zoom: 9.0,
            show_track_labels: false,
            show_labels: true,
            ..AdsbxBrowserOptions::default()
        };
        let output = browser.get_output(&options).unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-n737dw-labels.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
            zoom: 9.0,
            show_track_labels: false,
            show_labels: true,
            hide_infoblock: false,
            ..AdsbxBrowserOptions::default()
        };
        let output = browser.get_output(&options).unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-n737dw-labels-infoblock.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
            zoom: 9.0,
            layer: Some("esri".to_string()),
            ..AdsbxBrowserOptions::default()
        };
        let output = browser.get_output(&options).unwrap();
        let screenshot = output
            .screenshot
            .expect("Screenshot was requested but not generated");
        let mut file = std::fs::File::create("test-n737dw-esri.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
    }

    #[test]
    fn test_kml_download() {
        // Use a temporary directory for downloads
        let temp_dir = tempdir().expect("Failed to create temporary directory for KML test");
        let download_dir = temp_dir.path().to_path_buf();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()], // Use a known registration for potential KML content check
            history: Some(HistoryOptions::EntireDay(
                Utc.with_ymd_and_hms(2021, 12, 9, 0, 0, 0)
                    .unwrap()
                    .date_naive(),
            )),
            zoom: 10.0,
            request_screenshot: false, // Only request KML
            request_kml: Some(KmlType::Baro), // Request Barometric KML
            download_dir: download_dir.clone(),
            // Explicitly keep UI elements visible for KML download test
            hide_sidebar: false,
            hide_infoblock: false,
            ..Default::default()
        };

        let mut browser = AdsbxBrowser::new((800, 600)).unwrap();
        let output = browser.get_output(&options).unwrap();

        // Assert that screenshot is None (as it wasn't requested)
        assert!(output.screenshot.is_none(), "Screenshot was generated but not requested");

        // Assert that KML data is Some and validate it
        let kml_data = output.kml.expect("KML was requested but not generated");

        assert!(kml_data.filename.ends_with(".kml"), "Downloaded filename does not end with .kml");
        assert!(kml_data.data.contains("<kml"), "KML data does not contain <kml> tag");
        assert!(kml_data.data.contains("N737DW"), "KML data does not contain registration N737DW");
        println!("Successfully downloaded and validated KML file: {}", kml_data.filename);

        // temp_dir goes out of scope here, cleaning up the directory
    }
}

#[cfg(test)]
mod tests {
    use adsbx_screenshot::{AdsbxBrowser, AdsbxBrowserOptions, HistoryOptions};
    use chrono::prelude::*;

    #[test]
    fn test_reg_now() {
        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            zoom: 10.0,
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = AdsbxBrowser::new((800, 600))
            .unwrap()
            .screenshot(&options)
            .unwrap();
        let mut file = std::fs::File::create("test-maybe-n737dw-now.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
    }

    #[test]
    fn test_reg_history() {
        let mut browser = AdsbxBrowser::new((800, 600)).unwrap();
        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::TimeRange(
                Utc.ymd(2021, 12, 9).and_hms(0, 0, 0),
                Utc.ymd(2021, 12, 9).and_hms(1, 0, 0),
            )),
            zoom: 8.0,
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = browser.screenshot(&options).unwrap();
        let mut file = std::fs::File::create("test-n737dw-1-hour.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 9))),
            zoom: 8.0,
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = browser.screenshot(&options).unwrap();
        let mut file = std::fs::File::create("test-n737dw-all-day.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 9))),
            zoom: 9.0,
            show_track_labels: true,
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = browser.screenshot(&options).unwrap();
        let mut file = std::fs::File::create("test-n737dw-track-labels.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 9))),
            zoom: 9.0,
            show_track_labels: false,
            show_labels: true,
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = browser.screenshot(&options).unwrap();
        let mut file = std::fs::File::create("test-n737dw-labels.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 9))),
            zoom: 9.0,
            show_track_labels: false,
            show_labels: true,
            hide_infoblock: false,
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = browser.screenshot(&options).unwrap();
        let mut file = std::fs::File::create("test-n737dw-labels-infoblock.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();

        let options = AdsbxBrowserOptions {
            regs: vec!["N737DW".to_string()],
            history: Some(HistoryOptions::EntireDay(Utc.ymd(2021, 12, 9))),
            zoom: 9.0,
            layer: Some("esri".to_string()),
            ..AdsbxBrowserOptions::default()
        };
        let screenshot = browser.screenshot(&options).unwrap();
        let mut file = std::fs::File::create("test-n737dw-esri.png").unwrap();
        std::io::Write::write_all(&mut file, &screenshot.data).unwrap();
    }
}

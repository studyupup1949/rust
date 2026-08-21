use chrono::TimeZone;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;
use std::collections::HashSet;

fn check_filter_drops(record: &HashMap<String, String>, filters: &Vec<(String, String)>) -> bool {
    for (k, v) in filters {
        if record.contains_key(k) && record.get(k) == Some(v) {
            return true;
        }
    }

    false
}

fn check_filter_keeps(record: &HashMap<String, String>, filters: &Vec<(String, String)>) -> bool {
    for (k, v) in filters {
        if !record.contains_key(k) || record.get(k) != Some(v) {
            return false;
        }
    }

    true
}

fn check_time(record: &HashMap<String, String>, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
    if !record.contains_key("qso_date") || !record.contains_key("time_on") {
        return false;
    }
    let record_start = format!(
        "{}{}",
        record.get("qso_date").unwrap().clone(),
        record.get("time_on").unwrap().clone()
    );
    let naive_dt = NaiveDateTime::parse_from_str(&record_start, "%Y%m%d%H%M%S").unwrap();
    let record_start_utc = Utc.from_utc_datetime(&naive_dt);

    if start <= record_start_utc && record_start_utc <= end {
        return true;
    }

    false
}

fn insert_record(
    record: &HashMap<String, String>,
    inserts: &HashMap<String, String>,
) -> HashMap<String, String> {
    // TODO Should this cause an error if there is an existing key?
    // TODO This should be case insensitive
    let mut result = record.clone();
    result.extend(inserts.clone());

    result
}

fn replace_record(
    record: &HashMap<String, String>,
    replaces: &[(String, String)],
) -> HashMap<String, String> {
    // TODO Should this cause an error if there is no original key?
    // TODO This should be case insensitive
    let mut result = record.clone();

    result.extend(replaces.to_owned());

    result
}

fn delete_record(
    record: &HashMap<String, String>,
    deletes: &HashSet<String>,
) -> HashMap<String, String> {
    // TODO This should be case insensitive
    let mut result = record.clone();

    result.retain(|key, _value| !deletes.contains(key));

    result
}

pub fn process_drops(
    records: &[HashMap<String, String>],
    drop_args: Option<Vec<(String, String)>>,
) -> Vec<HashMap<String, String>> {
    let mut filtered_records: Vec<HashMap<String, String>> = Vec::new();

    let filters = drop_args.unwrap_or_default();

    for record in records.iter() {
        if !check_filter_drops(record, &filters) {
            filtered_records.push(record.clone());
        }
    }

    filtered_records
}

pub fn process_times(
    records: &[HashMap<String, String>],
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Vec<HashMap<String, String>> {
    let mut time_filtered_records: Vec<HashMap<String, String>> = Vec::new();

    let start = start.unwrap_or(DateTime::<Utc>::MIN_UTC);
    let end = end.unwrap_or(DateTime::<Utc>::MAX_UTC);

    for record in records.iter() {
        if check_time(record, start, end) {
            time_filtered_records.push(record.clone());
        }
    }

    time_filtered_records
}

pub fn process_keeps(
    records: &[HashMap<String, String>],
    keep_args: Option<Vec<(String, String)>>,
) -> Vec<HashMap<String, String>> {
    let mut kept_records = Vec::new();

    let filters = keep_args.unwrap_or_default();

    if !filters.is_empty() {
        for record in records.iter() {
            if check_filter_keeps(record, &filters) {
                kept_records.push(record.clone());
            }
        }

        return kept_records;
    }

    // If there are no things to keep, just pass the original back
    records.to_vec()
}

pub fn process_inserts(
    records: &[HashMap<String, String>],
    insert_args: Option<Vec<(String, String)>>,
) -> Vec<HashMap<String, String>> {
    let mut added_records = Vec::new();
    let inserts = insert_args.unwrap_or_default().into_iter().collect();

    for record in records.iter() {
        added_records.push(insert_record(record, &inserts));
    }

    added_records
}

pub fn process_replaces(
    records: &[HashMap<String, String>],
    replace_args: Option<Vec<(String, String)>>,
) -> Vec<HashMap<String, String>> {
    let mut replaced_records = Vec::new();
    let replaces = replace_args.unwrap_or_default();

    for record in records.iter() {
        replaced_records.push(replace_record(record, &replaces));
    }

    replaced_records
}

pub fn process_deletes(
    records: &[HashMap<String, String>],
    delete_args: Option<Vec<String>>,
) -> Vec<HashMap<String, String>> {
    let mut deleted_records = Vec::new();
    let deletes = delete_args.unwrap_or_default().into_iter().collect();

    for record in records.iter() {
        deleted_records.push(delete_record(record, &deletes));
    }

    deleted_records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let filters: Vec<(String, String)> = vec![];

        assert!(check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_empty_record_returns_false() {
        let record = HashMap::new();
        let filters = vec![("station_callsign".to_string(), "VA7XF".to_string())];

        assert!(!check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_matching_filter_returns_true() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let filters = vec![("station_callsign".to_string(), "VA7XF".to_string())];

        assert!(check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_non_matching_key_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let filters = vec![("mode".to_string(), "FT8".to_string())];

        assert!(!check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_matching_key_non_matching_value_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let filters = vec![("station_callsign".to_string(), "K6ABC".to_string())];

        assert!(!check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_multiple_filters_one_match_returns_true() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let filters = vec![
            ("mode".to_string(), "FT8".to_string()),
            ("gridsquare".to_string(), "CN89".to_string()),
            ("power".to_string(), "5W".to_string()),
        ];

        assert!(!check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_multiple_filters_no_match_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let filters = vec![
            ("mode".to_string(), "FT8".to_string()),
            ("gridsquare".to_string(), "DM04".to_string()),
            ("power".to_string(), "5W".to_string()),
        ];

        assert!(!check_filter_keeps(&record, &filters));
    }

    #[test]
    fn test_drops_empty_filter_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let filters: Vec<(String, String)> = vec![];

        assert!(!check_filter_drops(&record, &filters));
    }

    #[test]
    fn test_drops_empty_record_returns_false() {
        let record = HashMap::new();
        let filters = vec![("station_callsign".to_string(), "VA7XF".to_string())];

        assert!(!check_filter_drops(&record, &filters));
    }

    #[test]
    fn test_drops_matching_filter_returns_true() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let filters = vec![("station_callsign".to_string(), "VA7XF".to_string())];

        assert!(check_filter_drops(&record, &filters));
    }

    #[test]
    fn test_drops_non_matching_key_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let filters = vec![("mode".to_string(), "FT8".to_string())];

        assert!(!check_filter_drops(&record, &filters));
    }

    #[test]
    fn test_drops_matching_key_non_matching_value_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let filters = vec![("station_callsign".to_string(), "K6ABC".to_string())];

        assert!(!check_filter_drops(&record, &filters));
    }

    #[test]
    fn test_drops_multiple_filters_one_match_returns_true() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let filters = vec![
            ("mode".to_string(), "FT8".to_string()),
            ("gridsquare".to_string(), "CN89".to_string()),
            ("power".to_string(), "5W".to_string()),
        ];

        assert!(check_filter_drops(&record, &filters));
    }

    #[test]
    fn test_drops_multiple_filters_no_match_returns_false() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let filters = vec![
            ("mode".to_string(), "FT8".to_string()),
            ("gridsquare".to_string(), "DM04".to_string()),
            ("power".to_string(), "5W".to_string()),
        ];

        assert!(!check_filter_drops(&record, &filters));
    }

    // Helper function to create a record with date and time
    fn create_record(date: &str, time: &str) -> HashMap<String, String> {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), date.to_string());
        record.insert("time_on".to_string(), time.to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());
        record
    }

    #[test]
    fn test_missing_date_returns_false() {
        let mut record = HashMap::new();
        record.insert("time_on".to_string(), "140000".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap();

        assert!(!check_time(&record, start, end));
    }

    #[test]
    fn test_missing_time_returns_false() {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), "20250315".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap();

        assert!(!check_time(&record, start, end));
    }

    #[test]
    fn test_date_time_in_range_returns_true() {
        let record = create_record("20250315", "140000");

        let start = Utc.with_ymd_and_hms(2025, 3, 15, 13, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 3, 15, 15, 0, 0).unwrap();

        assert!(check_time(&record, start, end));
    }

    #[test]
    fn test_date_time_before_range_returns_false() {
        let record = create_record("20250315", "120000");

        let start = Utc.with_ymd_and_hms(2025, 3, 15, 13, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 3, 15, 15, 0, 0).unwrap();

        assert!(!check_time(&record, start, end));
    }

    #[test]
    fn test_date_time_after_range_returns_false() {
        let record = create_record("20250315", "160000");

        let start = Utc.with_ymd_and_hms(2025, 3, 15, 13, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 3, 15, 15, 0, 0).unwrap();

        assert!(!check_time(&record, start, end));
    }

    #[test]
    fn test_date_time_at_start_boundary_returns_true() {
        let record = create_record("20250315", "130000");

        let start = Utc.with_ymd_and_hms(2025, 3, 15, 13, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 3, 15, 15, 0, 0).unwrap();

        assert!(check_time(&record, start, end));
    }

    #[test]
    fn test_date_time_at_end_boundary_returns_true() {
        let record = create_record("20250315", "150000");

        let start = Utc.with_ymd_and_hms(2025, 3, 15, 13, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 3, 15, 15, 0, 0).unwrap();

        assert!(check_time(&record, start, end));
    }

    #[test]
    fn test_insert_into_empty_record() {
        let record = HashMap::new();
        let mut inserts = HashMap::new();
        inserts.insert("station_callsign".to_string(), "VA7XF".to_string());
        inserts.insert("gridsquare".to_string(), "CN89".to_string());

        let result = insert_record(&record, &inserts);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_insert_into_existing_record() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let mut inserts = HashMap::new();
        inserts.insert("gridsquare".to_string(), "CN89".to_string());
        inserts.insert("my_sig".to_string(), "POTA".to_string());

        let result = insert_record(&record, &inserts);

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
        assert_eq!(result.get("my_sig"), Some(&"POTA".to_string()));
    }

    #[test]
    fn test_insert_overwrites_existing_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let mut inserts = HashMap::new();
        inserts.insert("station_callsign".to_string(), "K6ABC".to_string());

        let result = insert_record(&record, &inserts);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"K6ABC".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_insert_empty_inserts() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let inserts = HashMap::new();

        let result = insert_record(&record, &inserts);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_insert_does_not_modify_original() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let mut inserts = HashMap::new();
        inserts.insert("gridsquare".to_string(), "CN89".to_string());

        let result = insert_record(&record, &inserts);

        // Check that the original record is unchanged
        assert_eq!(record.len(), 1);
        assert_eq!(record.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(record.get("gridsquare"), None);

        // Check that the original inserts are unchanged
        assert_eq!(inserts.len(), 1);
        assert_eq!(inserts.get("gridsquare"), Some(&"CN89".to_string()));
        assert_eq!(inserts.get("station_callsign"), None);

        // Check that the result has both keys
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_insert_multiple_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let mut inserts = HashMap::new();
        inserts.insert("gridsquare".to_string(), "CN89".to_string());
        inserts.insert("my_sig".to_string(), "POTA".to_string());
        inserts.insert("band".to_string(), "20m".to_string());
        inserts.insert("mode".to_string(), "SSB".to_string());

        let result = insert_record(&record, &inserts);

        assert_eq!(result.len(), 5);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
        assert_eq!(result.get("my_sig"), Some(&"POTA".to_string()));
        assert_eq!(result.get("band"), Some(&"20m".to_string()));
        assert_eq!(result.get("mode"), Some(&"SSB".to_string()));
    }

    #[test]
    fn test_replace_in_empty_record() {
        let record = HashMap::new();
        let replaces = vec![
            ("station_callsign".to_string(), "VA7XF".to_string()),
            ("gridsquare".to_string(), "CN89".to_string()),
        ];

        let result = replace_record(&record, &replaces);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_replace_existing_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let replaces = vec![("station_callsign".to_string(), "K6ABC".to_string())];

        let result = replace_record(&record, &replaces);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"K6ABC".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_replace_adds_new_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let replaces = vec![
            ("gridsquare".to_string(), "CN89".to_string()),
            ("my_sig".to_string(), "POTA".to_string()),
        ];

        let result = replace_record(&record, &replaces);

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
        assert_eq!(result.get("my_sig"), Some(&"POTA".to_string()));
    }

    #[test]
    fn test_replace_with_empty_vec() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());

        let replaces: Vec<(String, String)> = vec![];

        let result = replace_record(&record, &replaces);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_replace_does_not_modify_original() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let replaces = vec![
            ("station_callsign".to_string(), "K6ABC".to_string()),
            ("gridsquare".to_string(), "CN89".to_string()),
        ];

        let result = replace_record(&record, &replaces);

        // Check that the original record is unchanged
        assert_eq!(record.len(), 1);
        assert_eq!(record.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(record.get("gridsquare"), None);

        // Check that the result has both keys
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"K6ABC".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"CN89".to_string()));
    }

    #[test]
    fn test_replace_multiple_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("gridsquare".to_string(), "CN89".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let replaces = vec![
            ("station_callsign".to_string(), "K6ABC".to_string()),
            ("gridsquare".to_string(), "DM04".to_string()),
            ("band".to_string(), "20m".to_string()),
            ("mode".to_string(), "SSB".to_string()),
        ];

        let result = replace_record(&record, &replaces);

        assert_eq!(result.len(), 5);
        assert_eq!(result.get("station_callsign"), Some(&"K6ABC".to_string()));
        assert_eq!(result.get("gridsquare"), Some(&"DM04".to_string()));
        assert_eq!(result.get("my_sig"), Some(&"POTA".to_string()));
        assert_eq!(result.get("band"), Some(&"20m".to_string()));
        assert_eq!(result.get("mode"), Some(&"SSB".to_string()));
    }

    #[test]
    fn test_delete_record_empty() {
        let record = HashMap::new();
        let deletes = HashSet::new();
        let result = delete_record(&record, &deletes);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_delete_record_no_matches() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let mut deletes = HashSet::new();
        deletes.insert("other_key".to_string());

        let result = delete_record(&record, &deletes);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("my_sig"), Some(&"POTA".to_string()));
        assert_eq!(result.get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_delete_record_single_match() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let mut deletes = HashSet::new();
        deletes.insert("my_sig".to_string());

        let result = delete_record(&record, &deletes);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("station_callsign"), Some(&"VA7XF".to_string()));
        assert_eq!(result.get("band"), Some(&"20m".to_string()));
        assert!(result.get("my_sig").is_none());
    }

    #[test]
    fn test_delete_record_multiple_matches() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let mut deletes = HashSet::new();
        deletes.insert("station_callsign".to_string());
        deletes.insert("band".to_string());

        let result = delete_record(&record, &deletes);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("my_sig"), Some(&"POTA".to_string()));
        assert!(result.get("station_callsign").is_none());
        assert!(result.get("band").is_none());
    }

    #[test]
    fn test_delete_record_all_matches() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let mut deletes = HashSet::new();
        deletes.insert("station_callsign".to_string());
        deletes.insert("my_sig".to_string());
        deletes.insert("band".to_string());

        let result = delete_record(&record, &deletes);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_drops_empty_records() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        let drop_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_drops_no_filters() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let records = vec![record];
        let drop_args = None;

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_drops_no_match() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let records = vec![record];
        let drop_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_drops_match_single_record() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let drop_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_drops_match_some_records() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "20m".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "40m".to_string());

        let mut record3 = HashMap::new();
        record3.insert("station_callsign".to_string(), "K6ABC".to_string());
        record3.insert("band".to_string(), "20m".to_string());

        let records = vec![record1, record2, record3];
        let drop_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("station_callsign"), Some(&"W1AW".to_string()));
        assert_eq!(result[0].get("band"), Some(&"40m".to_string()));
    }

    #[test]
    fn test_process_drops_multiple_filters() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "20m".to_string());
        record1.insert("my_sig".to_string(), "POTA".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "40m".to_string());
        record2.insert("my_sig".to_string(), "SOTA".to_string());

        let records = vec![record1, record2];
        let drop_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("my_sig".to_string(), "SOTA".to_string()),
        ]);

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_drops_partial_matches() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "20m".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("my_sig".to_string(), "POTA".to_string());

        let records = vec![record1, record2];
        let drop_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("my_sig".to_string(), "POTA".to_string()),
        ]);

        let result = process_drops(&records, drop_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_times_empty_records() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_times_no_time_bounds() {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), "20230101".to_string());
        record.insert("time_on".to_string(), "000000".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];

        let result = process_times(&records, None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_times_record_within_bounds() {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), "20230101".to_string());
        record.insert("time_on".to_string(), "100000".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_times_record_before_start() {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), "20221231".to_string());
        record.insert("time_on".to_string(), "125959".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_times_record_after_end() {
        let mut record = HashMap::new();
        record.insert("time".to_string(), "2023-01-03T12:00:00Z".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_times_record_at_start_boundary() {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), "20230101".to_string());
        record.insert("time_on".to_string(), "000000".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_times_record_at_end_boundary() {
        let mut record = HashMap::new();
        record.insert("qso_date".to_string(), "20230102".to_string());
        record.insert("time_on".to_string(), "000000".to_string());
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_times_only_start_bound() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "W1AW".to_string());

        let mut record2 = HashMap::new();
        record2.insert("qso_date".to_string(), "20230101".to_string());
        record2.insert("time_on".to_string(), "100000".to_string());
        record2.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record1, record2];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());

        let result = process_times(&records, start, None);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_times_only_end_bound() {
        let mut record1 = HashMap::new();
        record1.insert("qso_date".to_string(), "20230101".to_string());
        record1.insert("time_on".to_string(), "100000".to_string());
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());

        let mut record2 = HashMap::new();
        record2.insert("qso_date".to_string(), "20230103".to_string());
        record2.insert("time_on".to_string(), "100000".to_string());
        record2.insert("station_callsign".to_string(), "W1AW".to_string());

        let records = vec![record1, record2];
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());

        let result = process_times(&records, None, end);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_times_multiple_records_mixed_results() {
        let mut record1 = HashMap::new();
        record1.insert("qso_date".to_string(), "20230101".to_string());
        record1.insert("time_on".to_string(), "100000".to_string());
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());

        let mut record2 = HashMap::new();
        record2.insert("qso_date".to_string(), "20221231".to_string());
        record2.insert("time_on".to_string(), "100000".to_string());
        record2.insert("station_callsign".to_string(), "W1AW".to_string());

        let mut record3 = HashMap::new();
        record3.insert("qso_date".to_string(), "20230105".to_string());
        record3.insert("time_on".to_string(), "100000".to_string());
        record3.insert("station_callsign".to_string(), "K6ABC".to_string());

        let records = vec![record1, record2, record3];
        let start = Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let end = Some(Utc.with_ymd_and_hms(2023, 1, 3, 0, 0, 0).unwrap());

        let result = process_times(&records, start, end);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_keeps_empty_records() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        let keep_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_keeps_no_filters() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let records = vec![record];
        let keep_args = None;

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_keeps_empty_filters() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let records = vec![record];
        let keep_args = Some(vec![]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_keeps_no_match() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());

        let records = vec![record];
        let keep_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_keeps_match_single_record() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let keep_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_keeps_match_some_records() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "20m".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "40m".to_string());

        let mut record3 = HashMap::new();
        record3.insert("station_callsign".to_string(), "K6ABC".to_string());
        record3.insert("band".to_string(), "20m".to_string());

        let records = vec![record1, record2, record3];
        let keep_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .any(|r| r.get("station_callsign") == Some(&"VA7XF".to_string()))
        );
        assert!(
            result
                .iter()
                .any(|r| r.get("station_callsign") == Some(&"K6ABC".to_string()))
        );
    }

    #[test]
    fn test_process_keeps_multiple_filters() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "20m".to_string());
        record1.insert("my_sig".to_string(), "POTA".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "40m".to_string());
        record2.insert("my_sig".to_string(), "SOTA".to_string());

        let records = vec![record1, record2];
        let keep_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("my_sig".to_string(), "POTA".to_string()),
        ]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_keeps_partial_match_fails() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let keep_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("my_sig".to_string(), "POTA".to_string()),
        ]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_keeps_filter_value_mismatch() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let keep_args = Some(vec![("band".to_string(), "40m".to_string())]);

        let result = process_keeps(&records, keep_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_inserts_empty_records() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        let insert_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_inserts_no_inserts() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let insert_args = None;

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_inserts_empty_inserts() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let insert_args = Some(vec![]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_inserts_new_key() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let insert_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_inserts_overwrite_existing() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "40m".to_string());

        let records = vec![record];
        let insert_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_inserts_multiple_records() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());

        let records = vec![record1, record2];
        let insert_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[1].get("station_callsign"), Some(&"W1AW".to_string()));
        assert_eq!(result[1].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_inserts_multiple_inserts() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let insert_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("my_sig".to_string(), "POTA".to_string()),
        ]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[0].get("my_sig"), Some(&"POTA".to_string()));
    }

    #[test]
    fn test_process_inserts_multiple_records_and_inserts() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("operator".to_string(), "John".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("my_sig".to_string(), "SOTA".to_string());

        let records = vec![record1, record2];
        let insert_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("mode".to_string(), "SSB".to_string()),
        ]);

        let result = process_inserts(&records, insert_args);
        assert_eq!(result.len(), 2);

        // Check first record
        assert_eq!(result[0].len(), 4);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("operator"), Some(&"John".to_string()));
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[0].get("mode"), Some(&"SSB".to_string()));

        // Check second record
        assert_eq!(result[1].len(), 4);
        assert_eq!(result[1].get("station_callsign"), Some(&"W1AW".to_string()));
        assert_eq!(result[1].get("my_sig"), Some(&"SOTA".to_string()));
        assert_eq!(result[1].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[1].get("mode"), Some(&"SSB".to_string()));
    }

    #[test]
    fn test_process_replaces_empty_records() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        let replace_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_replaces_no_replacements() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let replace_args = None;

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_replaces_empty_replacements() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let replace_args = Some(vec![]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_replaces_replace_existing() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "40m".to_string());

        let records = vec![record];
        let replace_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_replaces_nonexistent() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let replace_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_replaces_multiple_records() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "40m".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "80m".to_string());

        let records = vec![record1, record2];
        let replace_args = Some(vec![("band".to_string(), "20m".to_string())]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[1].get("station_callsign"), Some(&"W1AW".to_string()));
        assert_eq!(result[1].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_replaces_multiple_replaces() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "40m".to_string());
        record.insert("my_sig".to_string(), "SOTA".to_string());

        let records = vec![record];
        let replace_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("my_sig".to_string(), "POTA".to_string()),
        ]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[0].get("my_sig"), Some(&"POTA".to_string()));
    }

    #[test]
    fn test_process_replaces_multiple_records_and_replaces() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "40m".to_string());
        record1.insert("mode".to_string(), "CW".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "80m".to_string());
        record2.insert("mode".to_string(), "FT8".to_string());

        let records = vec![record1, record2];
        let replace_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("mode".to_string(), "SSB".to_string()),
        ]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 2);

        // Check first record
        assert_eq!(result[0].len(), 3);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[0].get("mode"), Some(&"SSB".to_string()));

        // Check second record
        assert_eq!(result[1].len(), 3);
        assert_eq!(result[1].get("station_callsign"), Some(&"W1AW".to_string()));
        assert_eq!(result[1].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[1].get("mode"), Some(&"SSB".to_string()));
    }

    #[test]
    fn test_process_replaces_mixed_existing_and_nonexistent() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "40m".to_string());

        let records = vec![record];
        let replace_args = Some(vec![
            ("band".to_string(), "20m".to_string()),
            ("mode".to_string(), "SSB".to_string()),
        ]);

        let result = process_replaces(&records, replace_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
        assert_eq!(result[0].get("mode"), Some(&"SSB".to_string()));
    }

    #[test]
    fn test_process_deletes_empty_records() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        let delete_args = Some(vec!["band".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_process_deletes_no_deletes() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let delete_args = None;

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_deletes_empty_deletes() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let delete_args = Some(vec![]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), Some(&"20m".to_string()));
    }

    #[test]
    fn test_process_deletes_single_key() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let delete_args = Some(vec!["band".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), None);
    }

    #[test]
    fn test_process_deletes_nonexistent_key() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());

        let records = vec![record];
        let delete_args = Some(vec!["band".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
    }

    #[test]
    fn test_process_deletes_multiple_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());
        record.insert("my_sig".to_string(), "POTA".to_string());
        record.insert("mode".to_string(), "SSB".to_string());

        let records = vec![record];
        let delete_args = Some(vec!["band".to_string(), "mode".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("my_sig"), Some(&"POTA".to_string()));
        assert_eq!(result[0].get("band"), None);
        assert_eq!(result[0].get("mode"), None);
    }

    #[test]
    fn test_process_deletes_multiple_records() {
        let mut record1 = HashMap::new();
        record1.insert("station_callsign".to_string(), "VA7XF".to_string());
        record1.insert("band".to_string(), "20m".to_string());
        record1.insert("mode".to_string(), "SSB".to_string());

        let mut record2 = HashMap::new();
        record2.insert("station_callsign".to_string(), "W1AW".to_string());
        record2.insert("band".to_string(), "40m".to_string());
        record2.insert("mode".to_string(), "CW".to_string());

        let records = vec![record1, record2];
        let delete_args = Some(vec!["band".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].len(), 2);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("mode"), Some(&"SSB".to_string()));
        assert_eq!(result[0].get("band"), None);

        assert_eq!(result[1].len(), 2);
        assert_eq!(result[1].get("station_callsign"), Some(&"W1AW".to_string()));
        assert_eq!(result[1].get("mode"), Some(&"CW".to_string()));
        assert_eq!(result[1].get("band"), None);
    }

    #[test]
    fn test_process_deletes_all_keys() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let delete_args = Some(vec!["station_callsign".to_string(), "band".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 0);
        assert!(result[0].is_empty());
    }

    #[test]
    fn test_process_deletes_mixed_existing_and_nonexistent() {
        let mut record = HashMap::new();
        record.insert("station_callsign".to_string(), "VA7XF".to_string());
        record.insert("band".to_string(), "20m".to_string());

        let records = vec![record];
        let delete_args = Some(vec!["band".to_string(), "mode".to_string()]);

        let result = process_deletes(&records, delete_args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0].get("station_callsign"),
            Some(&"VA7XF".to_string())
        );
        assert_eq!(result[0].get("band"), None);
    }
}

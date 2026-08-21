// ABOUTME: Normalizer to convert PiAware AircraftEntry data to domain AircraftObservation
// ABOUTME: Handles altitude preference, flight normalization, and raw JSON preservation

use crate::ingestion::piaware::AircraftEntry;
use crate::model::AircraftObservation;

/// Convert PiAware AircraftEntry to normalized AircraftObservation
#[allow(dead_code)] // Will be used in ingestion service (later prompts)
pub fn normalize_entry(ts_ms: i64, entry: &AircraftEntry) -> AircraftObservation {
    // Handle altitude preference: alt_geom over alt_baro
    let altitude = entry.alt_geom.or(entry.alt_baro);

    // Normalize flight callsign if present
    let flight = entry.flight.as_ref().map(|f| {
        let normalized = f.trim().to_uppercase();
        // Limit to 8 characters as per aviation standards
        if normalized.len() > 8 {
            normalized[..8].to_string()
        } else {
            normalized
        }
    });

    // Create raw JSON representation for debugging
    let raw_json = serde_json::to_string(entry).unwrap_or_else(|_| "{}".to_string());

    AircraftObservation {
        id: None,
        ts_ms,
        hex: entry.hex.clone().unwrap_or_default().to_uppercase(),
        flight,
        lat: entry.lat,
        lon: entry.lon,
        altitude,
        gs: entry.gs,
        rssi: entry.rssi,
        msg_count_total: entry.messages,
        raw_json,
        msg_rate_hz: None, // Will be computed later based on message counter deltas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_entry_complete_data() {
        let entry = AircraftEntry {
            hex: Some("ABC123".to_string()),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            alt_baro: Some(35000),
            alt_geom: Some(35100), // Should be preferred over alt_baro
            gs: Some(450.2),
            rssi: Some(-45.5),
            messages: Some(1000),
            seen: Some(0.1),
            seen_pos: Some(0.5),
            track: Some(90.0),
            squawk: Some("1200".to_string()),
            category: Some("A1".to_string()),
        };

        let obs = normalize_entry(1641024000000, &entry);

        assert_eq!(obs.ts_ms, 1641024000000);
        assert_eq!(obs.hex, "ABC123");
        assert_eq!(obs.flight, Some("UAL456".to_string()));
        assert_eq!(obs.lat, Some(40.7128));
        assert_eq!(obs.lon, Some(-74.0060));
        assert_eq!(obs.altitude, Some(35100)); // alt_geom preferred
        assert_eq!(obs.gs, Some(450.2));
        assert_eq!(obs.rssi, Some(-45.5));
        assert_eq!(obs.msg_count_total, Some(1000));
        assert_eq!(obs.msg_rate_hz, None); // Not computed yet
        assert!(!obs.raw_json.is_empty());
        assert!(obs.raw_json.contains("ABC123"));
    }

    #[test]
    fn test_normalize_entry_alt_baro_fallback() {
        let entry = AircraftEntry {
            hex: Some("DEF456".to_string()),
            flight: None,
            lat: None,
            lon: None,
            alt_baro: Some(28000),
            alt_geom: None, // Not available, should use alt_baro
            gs: None,
            rssi: None,
            messages: None,
            seen: None,
            seen_pos: None,
            track: None,
            squawk: None,
            category: None,
        };

        let obs = normalize_entry(1641024000000, &entry);

        assert_eq!(obs.altitude, Some(28000)); // alt_baro used as fallback
        assert_eq!(obs.hex, "DEF456");
        assert_eq!(obs.flight, None);
    }

    #[test]
    fn test_normalize_entry_flight_cleanup() {
        let test_cases = vec![
            (" ual456 ", "UAL456"),       // Trim and uppercase
            ("southwest123", "SOUTHWES"), // Truncate to 8 chars
            ("", ""),                     // Empty string
            ("x", "X"),                   // Single char
            ("TEST1234", "TEST1234"),     // Exactly 8 chars
        ];

        for (input, expected) in test_cases {
            let entry = AircraftEntry {
                hex: Some("TEST123".to_string()),
                flight: if input.is_empty() {
                    None
                } else {
                    Some(input.to_string())
                },
                lat: None,
                lon: None,
                alt_baro: None,
                alt_geom: None,
                gs: None,
                rssi: None,
                messages: None,
                seen: None,
                seen_pos: None,
                track: None,
                squawk: None,
                category: None,
            };

            let obs = normalize_entry(1641024000000, &entry);

            if input.is_empty() {
                assert_eq!(obs.flight, None);
            } else {
                assert_eq!(
                    obs.flight,
                    Some(expected.to_string()),
                    "Failed for input: '{}'",
                    input
                );
            }
        }
    }

    #[test]
    fn test_normalize_entry_missing_hex() {
        let entry = AircraftEntry {
            hex: None, // Missing hex
            flight: Some("TEST123".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            alt_baro: Some(35000),
            alt_geom: None,
            gs: Some(450.0),
            rssi: Some(-45.0),
            messages: Some(500),
            seen: Some(1.0),
            seen_pos: Some(2.0),
            track: Some(180.0),
            squawk: Some("7700".to_string()),
            category: Some("A3".to_string()),
        };

        let obs = normalize_entry(1641024000000, &entry);

        assert_eq!(obs.hex, ""); // Should default to empty string
        assert_eq!(obs.flight, Some("TEST123".to_string()));
        assert_eq!(obs.altitude, Some(35000));
    }

    #[test]
    fn test_normalize_entry_raw_json_populated() {
        let entry = AircraftEntry {
            hex: Some("ABC123".to_string()),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            alt_baro: Some(35000),
            alt_geom: None,
            gs: Some(450.2),
            rssi: Some(-45.5),
            messages: Some(1000),
            seen: Some(0.1),
            seen_pos: Some(0.5),
            track: Some(90.0),
            squawk: Some("1200".to_string()),
            category: Some("A1".to_string()),
        };

        let obs = normalize_entry(1641024000000, &entry);

        // Raw JSON should be valid JSON containing the entry data
        let parsed: serde_json::Value =
            serde_json::from_str(&obs.raw_json).expect("raw_json should be valid JSON");

        assert_eq!(parsed["hex"], "ABC123");
        assert_eq!(parsed["flight"], "UAL456");
        assert_eq!(parsed["lat"], 40.7128);
        assert_eq!(parsed["alt_baro"], 35000);
    }

    #[test]
    fn test_normalize_entry_minimal_data() {
        let entry = AircraftEntry {
            hex: Some("MIN123".to_string()),
            flight: None,
            lat: None,
            lon: None,
            alt_baro: None,
            alt_geom: None,
            gs: None,
            rssi: None,
            messages: None,
            seen: None,
            seen_pos: None,
            track: None,
            squawk: None,
            category: None,
        };

        let obs = normalize_entry(1641024000000, &entry);

        assert_eq!(obs.hex, "MIN123");
        assert_eq!(obs.flight, None);
        assert_eq!(obs.lat, None);
        assert_eq!(obs.lon, None);
        assert_eq!(obs.altitude, None);
        assert_eq!(obs.gs, None);
        assert_eq!(obs.rssi, None);
        assert_eq!(obs.msg_count_total, None);
        assert!(!obs.raw_json.is_empty());
    }
}

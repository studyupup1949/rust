//! Integration tests for journal name and abbreviation conversions.

mod common {
    use academic_journals::{get_abbreviation, get_full_name};

    #[test]
    fn empty_string_returns_none() {
        assert_eq!(get_abbreviation(""), None);
        assert_eq!(get_full_name(""), None);
    }

    #[test]
    fn whitespace_only_returns_none() {
        assert_eq!(get_abbreviation("   "), None);
        assert_eq!(get_full_name("   "), None);
    }

    #[test]
    fn nonexistent_journal_returns_none() {
        assert_eq!(get_abbreviation("Nonexistent Journal XYZ"), None);
        assert_eq!(get_full_name("Nonexistent Abbr XYZ"), None);
    }

    #[test]
    fn case_sensitivity_is_enforced() {
        // A journal name only matches in its correct case.
        // "critical care medicine" should not match "Critical Care Medicine".
        assert_eq!(get_abbreviation("critical care medicine"), None);
        assert_eq!(get_full_name("crit care med"), None);
    }
}

// When both dot and dotless are active, build.rs prioritizes dot. Only run
// dotless-specific tests when dotless is enabled and dot is not.
#[cfg(all(feature = "dotless", not(feature = "dot")))]
mod dotless {
    use academic_journals::{get_abbreviation, get_full_name};

    #[test]
    fn data_is_loaded() {
        assert!(
            get_abbreviation("Critical Care Medicine").is_some(),
            "Dotless journal data appears empty or not loaded"
        );
    }

    #[test]
    fn abbreviation_for_critical_care_medicine() {
        assert_eq!(
            get_abbreviation("Critical Care Medicine"),
            Some("Crit Care Med".to_string())
        );
    }

    #[test]
    fn abbreviation_for_academic_emergency_medicine() {
        assert_eq!(
            get_abbreviation("Academic Emergency Medicine"),
            Some("Acad Emerg Med".to_string())
        );
    }

    #[test]
    fn get_full_name_for_crit_care_med() {
        assert_eq!(
            get_full_name("Crit Care Med"),
            Some("Critical Care Medicine".to_string())
        );
    }

    #[test]
    fn roundtrip_critical_care_medicine() {
        let full_name = "Critical Care Medicine";
        let abbr = get_abbreviation(full_name).expect("abbreviation should exist");
        let recovered = get_full_name(&abbr).expect("full name should round-trip");
        assert_eq!(recovered, full_name);
    }
}

#[cfg(feature = "dot")]
mod dot {
    use academic_journals::{get_abbreviation, get_full_name};

    #[test]
    fn data_is_loaded() {
        assert!(
            get_abbreviation("ACS Catalysis").is_some(),
            "Dot journal data appears empty or not loaded"
        );
    }

    #[test]
    fn abbreviation_for_acs_catalysis() {
        assert_eq!(
            get_abbreviation("ACS Catalysis"),
            Some("ACS Catal.".to_string())
        );
    }

    #[test]
    fn abbreviation_for_acs_applied_materials() {
        assert_eq!(
            get_abbreviation("ACS Applied Materials & Interfaces"),
            Some("ACS Appl. Mater. Interfaces".to_string())
        );
    }

    #[test]
    fn get_full_name_for_acs_catal() {
        assert_eq!(
            get_full_name("ACS Catal."),
            Some("ACS Catalysis".to_string())
        );
    }

    #[test]
    fn roundtrip_acs_catalysis() {
        let full_name = "ACS Catalysis";
        let abbr = get_abbreviation(full_name).expect("abbreviation should exist");
        let recovered = get_full_name(&abbr).expect("full name should round-trip");
        assert_eq!(recovered, full_name);
    }
}

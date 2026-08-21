//! Property-based tests for the diff engine and ignore rules.

use proptest::prelude::*;
use super::ignore::IgnoreRules;

proptest! {
    #[test]
    fn ignore_rules_never_panic(text in ".*", path in "[a-zA-Z0-9/._-]{0,50}") {
        // Parsing arbitrary text as ignore rules should not panic.
        if let Ok(rules) = IgnoreRules::from_str(&text) {
            let _ = rules.is_ignored(&path);
        }
    }

    #[test]
    fn empty_rules_ignore_nothing(path in "[a-zA-Z0-9/._-]{1,40}") {
        let rules = IgnoreRules::from_str("").unwrap();
        prop_assert!(!rules.is_ignored(&path),
            "Empty ruleset should not ignore any path");
    }

    #[test]
    fn star_glob_matches_flat_file(
        dir in "[a-z]{1,8}",
        ext in "[a-z]{1,4}",
        name in "[a-z0-9]{1,8}"
    ) {
        let pattern = format!("{dir}/*.{ext}");
        let path    = format!("{dir}/{name}.{ext}");
        let rules   = IgnoreRules::from_str(&pattern).unwrap();
        prop_assert!(rules.is_ignored(&path),
            "Pattern {pattern:?} should match path {path:?}");
    }

    #[test]
    fn negation_unignores(ext in "[a-z]{1,4}", name in "[a-z]{1,8}") {
        // Pattern: ignore all .ext, then un-ignore name.ext
        let text = format!("*.{ext}\n!{name}.{ext}");
        let rules = IgnoreRules::from_str(&text).unwrap();
        prop_assert!(!rules.is_ignored(&format!("{name}.{ext}")),
            "Negation should un-ignore the specific file");
    }
}

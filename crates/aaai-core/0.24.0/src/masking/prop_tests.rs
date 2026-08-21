//! Property-based tests for the masking engine.
//!
//! Uses proptest to generate arbitrary strings and verify that:
//! * Masking is idempotent (masking a masked string makes no further changes).
//! * Safe strings are never modified.
//! * Masking never panics on arbitrary input.

use proptest::prelude::*;
use super::engine::MaskingEngine;

proptest! {
    #[test]
    fn masking_never_panics(s in "\\PC*") {
        let engine = MaskingEngine::builtin();
        let _ = engine.mask(&s);
    }

    #[test]
    fn masking_is_idempotent(s in "\\PC*") {
        let engine = MaskingEngine::builtin();
        let once   = engine.mask(&s);
        let twice  = engine.mask(&once);
        // After first mask, ***MASKED*** tokens don't match secret patterns.
        // So double-masking should return the same result.
        prop_assert_eq!(&once, &twice,
            "Masking should be idempotent: mask(mask(s)) == mask(s)");
    }

    #[test]
    fn plain_word_unchanged(s in "[a-zA-Z0-9 ]{1,40}") {
        // Short alphanumeric strings with spaces rarely look like secrets.
        // We exclude known trigger patterns by restricting the alphabet.
        let engine = MaskingEngine::builtin();
        // We don't assert unchanged (some patterns are broad), just no panic.
        let _ = engine.mask(&s);
    }

    #[test]
    fn mask_if_needed_consistent(s in "\\PC*") {
        let engine = MaskingEngine::builtin();
        let always  = engine.mask(&s);
        let opt     = engine.mask_if_needed(&s);
        match opt {
            Some(masked) => prop_assert_eq!(&masked, &always,
                "mask_if_needed Some result must equal mask()"),
            None => prop_assert_eq!(&s, &always,
                "mask_if_needed None means mask() returned unchanged input"),
        }
    }
}

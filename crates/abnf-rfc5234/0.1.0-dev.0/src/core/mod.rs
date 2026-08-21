//! Core ABNF
//!
//! > Basic rules that are in common use. [...] Note that these rules are only valid
//! > for ABNF encoded in 7-bit ASCII or in characters sets that are a
//! > superset of 7-bit ASCII.
//!
//! <https://www.rfc-editor.org/rfc/rfc5234.html#appendix-B>

/// RFC5234 Core Rules
/// 
/// <https://www.rfc-editor.org/rfc/rfc5234.html#appendix-B.1>
pub mod rules {
    use pest_derive::Parser;
    use pest::Parser;
    
    /// Tests for Core Rule "ALPHA".
    /// 
    /// `ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z`
    pub mod alpha {
        use super::Parser;

        #[derive(Parser)]
        #[grammar = "core/alpha.pest"]
        pub struct ParserSingleAlpha;
        
        /// Tests that ALPHA example value `0x41` ("A") is successfully parsed
        #[test]
        fn positive_single_alpha_0x41() {
            let mut pairs = ParserSingleAlpha::parse(Rule::alpha, "A").expect("alpha 'A'");
            let pair = pairs.next().expect("parsed pairs has a pair");
            assert_eq!(Rule::alpha, pair.as_rule());
            assert_eq!("A", pair.as_str());
            assert!(pairs.next().is_none());
        }

        /// Tests that non-ALPHA example value `0x40` ("@") is rejected
        #[test]
        fn negative_single_non_alpha_0x40() {
            let e = ParserSingleAlpha::parse(Rule::alpha, "@").expect_err("non-alpha '@'");
            assert_eq!(e.location, pest::error::InputLocation::Pos(0));
        }

        /// Tests that ALPHA example value `0x7a` ("z") is successfully parsed
        #[test]
        fn positive_single_alpha_0x7a() {
            let mut pairs = ParserSingleAlpha::parse(Rule::alpha, "z").expect("alpha 'z'");
            let pair = pairs.next().expect("parsed pairs has a pair");
            assert_eq!(Rule::alpha, pair.as_rule());
            assert_eq!(&[0x7a], pair.as_str().as_bytes());
            assert!(pairs.next().is_none());
        }

        /// Tests that non-ALPHA example value `0x7b` ("{") is rejected
        #[test]
        fn negative_single_non_alpha_0x7b() {
            let e = ParserSingleAlpha::parse(Rule::alpha, "{").expect_err("non-alpha '{'");
            assert_eq!(e.location, pest::error::InputLocation::Pos(0));
        }
    }
}
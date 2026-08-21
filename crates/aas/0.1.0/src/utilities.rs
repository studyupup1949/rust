use oxilangtag::LanguageTag;
use serde::{Deserialize, Deserializer};

/// check if all chars of the text are valid using
/// the regex for text from the AAS Spec as the baseline.
pub fn validate_text(txt: &str) -> bool {
    txt.chars().all(|c| {
        matches!(c,
            '\t'                            // horizontal tab
            | '\n'                          // Line feed
            | '\r'                          // carriage return
            | ' '..='\u{7F}'                // visible ascii
            | '\u{E000}'..= '\u{FFFD}'      // BMP after surrogates and private use
            | '\u{10000}'..='\u{10FFFF}'    // Supplementary planes
        )
    })
}

/// deserialize AND validate text
pub fn deserialize_normalized_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    let valid = validate_text(&buf);

    match valid {
        true => Ok(buf),
        false => Err(serde::de::Error::custom(
            "Non valid character (control ones) found.",
        )),
    }
}

/// only valid language tags, make tag normalized (EN => en, EN-US => en-US, en-us => en-US,...).
pub fn deserialize_normalized_lang_tag<'de, D>(
    deserializer: D,
) -> Result<LanguageTag<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    LanguageTag::parse_and_normalize(&buf).map_err(serde::de::Error::custom)
}

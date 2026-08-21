use crate::part1::v3_1::primitives::Identifier;
use oxilangtag::LanguageTag;
use serde::{Deserialize, Deserializer};

/// check if all chars of the text are valid using
/// the regex for text from the AAS Spec as the baseline.
/// see https://industrialdigitaltwin.io/aas-specifications/IDTA-01001/v3.1.2/spec-metamodel/constraints.html#constraints-for-types
pub fn validate_text(txt: &str) -> bool {
    txt.chars().all(|c| {
        matches!(c,
            '\t'                            // horizontal tab
            | '\n'                          // Line feed
            | '\r'                          // carriage return
            | ' '..='\u{D7FF}'              //  all the characters of the Basic Multilingual Plane, and
            | '\u{10000}'..='\u{10FFFF}'    // all the characters beyond the Basic Multilingual Plane (e.g., emoticons).
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
        false => Err(serde::de::Error::custom(format!(
            "(Normalized Text) Non valid character (control ones) found, found '{}'",
            &buf
        ))),
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

// "" => None for identifiers.
pub fn deserialize_empty_identifier_as_none<'de, D>(
    deserializer: D,
) -> Result<Option<Identifier>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;

    match o {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Identifier::try_from(s)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

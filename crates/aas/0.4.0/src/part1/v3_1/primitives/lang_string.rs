use crate::utilities::{
    deserialize_normalized_lang_tag, deserialize_normalized_text, validate_text,
};
use oxilangtag::{LanguageTag, LanguageTagParseError};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct LangString {
    #[serde(deserialize_with = "deserialize_normalized_lang_tag")]
    #[cfg_attr(feature = "openapi", schema(value_type = String, example = "en-EN"))]
    pub language: LanguageTag<String>,

    #[serde(deserialize_with = "deserialize_normalized_text")]
    pub text: String,
}

impl Default for LangString {
    fn default() -> Self {
        Self {
            language: LanguageTag::parse_and_normalize("en").unwrap(),
            text: String::default(),
        }
    }
}

#[derive(Error, Debug)]
pub enum LangStringParseRDFError {
    #[error("The format of the RDF/Turtle syntax is wrong and cannot be parsed")]
    IncorrectFormat,

    #[error("The Text part does contain non valid characters specified by the AAS spec")]
    NonValidCharacters,

    #[error("Language tag couldn't be parsed")]
    ParseError(#[from] LanguageTagParseError),
}

impl LangString {
    pub fn try_new(language: &str, text: String) -> Result<Self, LanguageTagParseError> {
        let language = LanguageTag::parse_and_normalize(language)?;
        Ok(Self { language, text })
    }

    /// Returns String in the RDF format "Text@TAG"
    /// i.e. "Speed"@en
    pub fn to_string(&self) -> String {
        format!(r#""{}"@{}"#, self.text, self.language.to_string())
    }
}

impl FromStr for LangString {
    type Err = LangStringParseRDFError;

    /// Parse from RDF/Turtle syntax like `"Hello"@en`
    /// It supports normalized and non normalized strings, but normalizes them after.  
    ///  
    /// # Example
    /// ```
    /// use std::str::FromStr;
    /// use oxilangtag::LanguageTag;
    /// use aas::part1::v3_1::LangString;
    ///
    /// let expected = LangString::try_new("EN", "Speed".to_string()).unwrap();
    /// let actual = LangString::from_str(r#""Speed"@EN"#).ok().unwrap();
    ///
    /// assert_eq!(actual, expected);
    /// assert_eq!(actual.to_string(), r#""Speed"@en"#);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang_string: Vec<&str> = s.split('@').collect();

        if lang_string.len() != 2 {
            return Err(LangStringParseRDFError::IncorrectFormat);
        }

        let mut text = String::from(lang_string[0]);

        // Question: Build Proper parser ?
        // "...", the double quotes are important.
        if text.chars().count() < 2 {
            return Err(LangStringParseRDFError::IncorrectFormat);
        }

        // remove the quotes
        let (q1, q2) = (text.remove(0), text.remove(text.len() - 1));

        // test if right chars are removed
        if q1 != '\"' || q2 != '\"' {
            return Err(LangStringParseRDFError::IncorrectFormat);
        }

        if !validate_text(&text) {
            return Err(LangStringParseRDFError::NonValidCharacters);
        }

        let language = LanguageTag::parse_and_normalize(lang_string[1])
            .map_err(LangStringParseRDFError::ParseError)?;

        Ok(LangString { language, text })
    }
}

// todo more tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turtle_syntax() {
        let expected = LangString::try_new("EN", "Sample text".to_string()).unwrap();

        let turtle_parsed = LangString::from_str(r#""Sample text"@EN"#).ok().unwrap();

        // assert
        assert_eq!(turtle_parsed, expected);
    }

    #[test]
    fn turtle_syntax_empty_text() {
        let expected = LangString::try_new("EN", "".to_string()).unwrap();

        let turtle_parsed = LangString::from_str(r#"""@EN"#).ok().unwrap();

        // assert
        assert_eq!(turtle_parsed, expected);
    }

    /// Cannot test for type, due to not being able to implement PartialEq.
    #[test]
    #[should_panic]
    fn turtle_syntax_no_text_no_quotes() {
        LangString::from_str(r#"@EN"#).unwrap();
    }

    #[test]
    fn test_deserialize() {
        let json = r#"
        {
            "language": "EN",
            "text": "Sample test text"
        }"#;

        let expected = LangString::try_new("EN", "Sample test text".to_string()).unwrap();

        let deserialized = serde_json::from_str(json).expect("Should deserialize");

        assert_eq!(expected, deserialized);
    }
}

use serde::{Deserialize, Serialize};

use crate::{Iri, Map, Name, impl_default};

/// Represents the `@context` field used by ActivityStream JSON-LD types.
///
/// # Example (with string)
///
/// ```rust
/// use activitystreams_vocabulary::Note;
///
/// # fn main() {
/// let summary = "A note";
/// let content = "My dog has fleas.";
/// let json_str = format!(
/// r#"{{
///   "@context": "https://www.w3.org/ns/activitystreams",
///   "type": "Note",
///   "summary": "{summary}",
///   "content": "{content}"
/// }}"#);
///
/// let note = Note::new().with_summary(summary).with_content(content);
///
/// assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<Note>(&json_str).unwrap(),
///     note,
/// );
/// # }
/// ```
///
/// # Example (with object)
///
/// ```rust
/// use activitystreams_vocabulary::{Context, Name, Note};
///
/// # fn main() {
/// let summary = "A note";
/// let content = "My dog has fleas.";
///
/// let vocab_key = Name::try_from("@vocab").unwrap();
/// let vocab_val = "https://www.w3.org/ns/activitystreams#";
///
/// let ext_key = Name::try_from("ext").unwrap();
/// let ext_val = "https://canine-extension.example/terms/";
///
/// let language_key = Name::try_from("@language").unwrap();
/// let language_val = "en";
///
/// let json_str = format!(
/// r#"{{
///   "@context": {{
///     "{language_key}": "{language_val}",
///     "{vocab_key}": "{vocab_val}",
///     "{ext_key}": "{ext_val}"
///   }},
///   "type": "Note",
///   "summary": "{summary}",
///   "content": "{content}",
///   "ext:nose": 0,
///   "ext:smell": "terrible"
/// }}"#);
///
/// let extras = [
///     (String::from("ext:nose"), serde_json::Value::Number(0u64.into())),
///     (String::from("ext:smell"), serde_json::Value::String("terrible".into())),
/// ];
///
/// let context = Context::map([
///     (vocab_key, vocab_val),
///     (ext_key, ext_val),
///     (language_key, language_val),
/// ]);
///
/// let note = Note::new()
///     .with_context_property(context)
///     .with_summary(summary)
///     .with_content(content)
///     .with_extra_fields(extras);
///
/// assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<Note>(&json_str).unwrap(),
///     note,
/// );
/// # }
/// ```
///
/// # Example (with array)
///
/// ```rust
/// use activitystreams_vocabulary::{Context, Note};
///
/// # fn main() {
/// let context_uri = Context::URI;
/// let context_key = "css";
/// let context_val = "http://www.w3.org/ns/oa#styledBy";
///
/// let summary = "A note";
/// let content = "My dog has fleas.";
///
/// let json_str = format!(
/// r#"{{
///   "@context": [
///     "{context_uri}",
///     {{
///       "{context_key}": "{context_val}"
///     }}
///   ],
///   "type": "Note",
///   "summary": "{summary}",
///   "content": "{content}"
/// }}"#);
///
/// let context_obj = [(context_key, context_val)]
///     .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.into())));
///
/// let context = Context::array([
///     serde_json::Value::String(context_uri.into()),
///     serde_json::Value::Object(context_obj.into_iter().collect()),
/// ]);
///
/// let note = Note::new()
///     .with_context_property(context)
///     .with_summary(summary)
///     .with_content(content);
///
/// assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<Note>(&json_str).unwrap(),
///     note,
/// );
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Context {
    Iri(Iri),
    Array(Vec<serde_json::Value>),
    Map(Map<Name, serde_json::Value>),
}

impl Context {
    /// Represents the JSON-LD namespace URI for ActivityStreams types.
    pub const URI: &str = "https://www.w3.org/ns/activitystreams";

    /// Creates a new [Context].
    pub fn new() -> Self {
        Self::Iri(Iri::new_trusted(Self::URI.into()))
    }

    /// Creates a new [Context] with a JSON-LD map representation.
    pub fn map<N, V, M>(map: M) -> Self
    where
        N: Into<Name>,
        V: Into<serde_json::Value>,
        M: IntoIterator<Item = (N, V)>,
    {
        Self::Map(map.into_iter().map(|(n, v)| (n.into(), v.into())).collect())
    }

    /// Creates a new [Context] with a JSON-LD array representation.
    pub fn array<V, A>(array: A) -> Self
    where
        V: Into<serde_json::Value>,
        A: IntoIterator<Item = V>,
    {
        Self::Array(array.into_iter().map(|v| v.into()).collect())
    }
}

impl_default!(Context);

impl core::fmt::Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Iri(iri) => write!(f, "{iri}"),
            Self::Array(arr) => serde_json::to_string(arr)
                .map_err(|_| core::fmt::Error)
                .and_then(|a| write!(f, "{a}")),
            Self::Map(map) => serde_json::to_string(map)
                .map_err(|_| core::fmt::Error)
                .and_then(|m| write!(f, "{m}")),
        }
    }
}

impl From<Context> for String {
    fn from(val: Context) -> Self {
        val.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Note;

    #[test]
    fn test_context_string() {
        let summary = "A note";
        let content = "My dog has fleas.";
        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "summary": "{summary}",
  "content": "{content}"
}}"#
        );

        let note = Note::new().with_summary(summary).with_content(content);

        assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
        assert_eq!(serde_json::from_str::<Note>(&json_str).unwrap(), note,);
    }

    #[test]
    fn test_context_map() {
        let summary = "A note";
        let content = "My dog has fleas.";

        let vocab_key = Name::try_from("@vocab").unwrap();
        let vocab_val = "https://www.w3.org/ns/activitystreams#";

        let ext_key = Name::try_from("ext").unwrap();
        let ext_val = "https://canine-extension.example/terms/";

        let language_key = Name::try_from("@language").unwrap();
        let language_val = "en";

        let json_str = format!(
            r#"{{
  "@context": {{
    "{language_key}": "{language_val}",
    "{vocab_key}": "{vocab_val}",
    "{ext_key}": "{ext_val}"
  }},
  "type": "Note",
  "summary": "{summary}",
  "content": "{content}",
  "ext:nose": 0,
  "ext:smell": "terrible"
}}"#
        );

        let extras = [
            (
                String::from("ext:nose"),
                serde_json::Value::Number(0u64.into()),
            ),
            (
                String::from("ext:smell"),
                serde_json::Value::String("terrible".into()),
            ),
        ];

        let context = Context::map([
            (vocab_key, vocab_val),
            (ext_key, ext_val),
            (language_key, language_val),
        ]);

        let note = Note::new()
            .with_context_property(context)
            .with_summary(summary)
            .with_content(content)
            .with_extra_fields(extras);

        assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
        assert_eq!(serde_json::from_str::<Note>(&json_str).unwrap(), note,);
    }

    #[test]
    fn test_context_array() {
        let context_uri = Context::URI;
        let context_key = "css";
        let context_val = "http://www.w3.org/ns/oa#styledBy";

        let summary = "A note";
        let content = "My dog has fleas.";

        let json_str = format!(
            r#"{{
  "@context": [
    "{context_uri}",
    {{
      "{context_key}": "{context_val}"
    }}
  ],
  "type": "Note",
  "summary": "{summary}",
  "content": "{content}"
}}"#
        );

        let context_obj = [(context_key, context_val)]
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.into())));

        let context = Context::array([
            serde_json::Value::String(context_uri.into()),
            serde_json::Value::Object(context_obj.into_iter().collect()),
        ]);

        let note = Note::new()
            .with_context_property(context)
            .with_summary(summary)
            .with_content(content);

        assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
        assert_eq!(serde_json::from_str::<Note>(&json_str).unwrap(), note,);
    }
}

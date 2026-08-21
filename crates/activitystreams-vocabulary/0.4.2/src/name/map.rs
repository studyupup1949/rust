use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{LanguageTag, field_access, impl_default, impl_display};

use super::Name;

/// A simple, human-readable, plain-text name for an object, expressed as language-tagged values..
///
/// HTML markup **MUST NOT** be included.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct NameMap {
    #[serde(flatten)]
    map: BTreeMap<LanguageTag, Name>,
}

impl NameMap {
    /// Creates a new [NameMap].
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

field_access! {
    NameMap {
        /// Represents the language-tagged name values.
        ///
        /// HTML markup **MUST NOT** be included.
        map: as_ref { BTreeMap<LanguageTag, Name> },
    }
}

impl_default!(NameMap);
impl_display!(NameMap, json);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_map() {
        let en_tag = LanguageTag::try_from("en").unwrap();
        let es_tag = LanguageTag::try_from("es").unwrap();
        let zh_tag = LanguageTag::try_from("zh-Hans").unwrap();

        let name_en: Name = "A simple note".try_into().unwrap();
        let name_es: Name = "Una nota sencilla".try_into().unwrap();
        let name_zh: Name = "一段简单的笔记".try_into().unwrap();

        let name_str =
            format!(r#"{{"{en_tag}":"{name_en}","{es_tag}":"{name_es}","{zh_tag}":"{name_zh}"}}"#);

        let name_map =
            NameMap::new().with_map([(en_tag, name_en), (es_tag, name_es), (zh_tag, name_zh)]);

        assert_eq!(
            serde_json::from_str::<NameMap>(&name_str).unwrap(),
            name_map
        );
        assert_eq!(serde_json::to_string(&name_map).unwrap(), name_str);
    }

    #[test]
    fn test_invalid_name_map() {
        let en_tag = LanguageTag::try_from("en").unwrap();
        let es_tag = LanguageTag::try_from("es").unwrap();
        let zh_tag = LanguageTag::try_from("zh-Hans").unwrap();

        let invalid_name_en = "A simple <em>note</em>";
        let invalid_name_es = "Una <em>nota</em> sencilla";
        let invalid_name_zh = "一段<em>简单的</em>笔记";

        let invalid_name_str = format!(
            r#"{{"{en_tag}":"{invalid_name_en}","{es_tag}":"{invalid_name_es}","{zh_tag}":"{invalid_name_zh}"}}"#
        );

        assert!(serde_json::from_str::<NameMap>(&invalid_name_str).is_err());
    }
}

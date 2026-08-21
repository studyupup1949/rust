use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{LanguageTag, field_access, impl_default, impl_display};

/// Multiple language-tagged values.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct LanguageMap {
    #[serde(flatten)]
    map: BTreeMap<LanguageTag, String>,
}

impl LanguageMap {
    /// Creates a new [LanguageMap].
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Gets a mutable reference to the map.
    pub fn map_mut(&mut self) -> &mut BTreeMap<LanguageTag, String> {
        &mut self.map
    }
}

field_access! {
    LanguageMap {
        /// Inner mapping of [LanguageTag] to content string in that language.
        map: as_ref { BTreeMap<LanguageTag, String> },
    }
}

impl_default!(LanguageMap);
impl_display!(LanguageMap, json);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_map() {
        let en_tag = LanguageTag::try_from("en").unwrap();
        let es_tag = LanguageTag::try_from("es").unwrap();
        let zh_tag = LanguageTag::try_from("zh-Hans").unwrap();

        let summary_en = "A simple <em>note</em>";
        let summary_es = "Una <em>nota</em> sencilla";
        let summary_zh = "一段<em>简单的</em>笔记";

        let summary_str = format!(
            r#"{{"{en_tag}":"{summary_en}","{es_tag}":"{summary_es}","{zh_tag}":"{summary_zh}"}}"#
        );

        let summary = LanguageMap::new().with_map([
            (en_tag, summary_en.to_owned()),
            (es_tag, summary_es.to_owned()),
            (zh_tag, summary_zh.to_owned()),
        ]);

        assert_eq!(serde_json::to_string(&summary).unwrap(), summary_str);
        assert_eq!(
            serde_json::from_str::<LanguageMap>(&summary_str).unwrap(),
            summary
        );
    }
}

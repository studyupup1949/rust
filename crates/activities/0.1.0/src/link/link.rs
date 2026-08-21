use crate::link::{LinkType, Mime};
use isolang::Language;
use serde::{Deserialize, Serialize};

use crate::link::Url;

use derive_builder::Builder;
use std::convert::TryFrom;

const LINK_TYPE: &'static str = "Link";

#[derive(Serialize, Deserialize, Builder, Clone)]
pub struct Link {
    // #[serde(rename="type")]
    r#type: String,
    href: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_type: Option<Mime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href_lang: Option<Language>,
}

impl Default for Link {
    fn default() -> Self {
        Self {
            r#type: LINK_TYPE.to_string(),
            href: Url::new("https://dbhattarai.info.np".parse().unwrap()),
            media_type: None,
            name: None,
            rel: None,
            href_lang: None,
        }
    }
}

impl TryFrom<&str> for Link {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let url: url::Url = value.parse().unwrap();
        let href = Url::new(url);
        Ok(Self {
            href,
            ..Default::default()
        })
    }
}
impl LinkType for Link {
    fn href(&self) -> &Url {
        &self.href
    }

    fn media_type(&self) -> Option<&Mime> {
        match &self.media_type {
            Some(val) => Some(val),
            _ => None,
        }
    }

    fn name(&self) -> Option<&String> {
        match &self.name {
            Some(val) => Some(val),
            _ => None,
        }
    }

    fn rel(&self) -> Option<&String> {
        match &self.rel {
            Some(val) => Some(val),
            _ => None,
        }
    }

    fn href_lang(&self) -> Option<&Language> {
        match &self.href_lang {
            Some(val) => Some(val),
            _ => None,
        }
    }
}

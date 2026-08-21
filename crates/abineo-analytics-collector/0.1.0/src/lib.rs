//! Used in the backend of the Abineo Analytics server.
//!
//! See [api functions] for entry points.
//!
//! [api functions]: api#functions

use crate::api::PubVisitor;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use serde_json::Value;
use uaparser::{Parser, UserAgentParser};
use url::Url;

pub mod api;
pub mod hash;

use crate::hash::Hasher;

include!(concat!(env!("OUT_DIR"), "/timezone-codegen.rs"));

static UAP_REGEXES: &[u8] = include_bytes!("../uap-core/regexes.yaml");

lazy_static! {
    pub static ref UA_PARSER: UserAgentParser =
        UserAgentParser::from_bytes(UAP_REGEXES).expect("can parse regexes.yaml");
}

#[derive(Debug, Default, Clone)]
pub struct Visitor {
    pub id: i64,
    pub project: i64,
    pub region: Option<String>,
    pub timezone: String,
    pub language: String,
    pub browser: Option<String>,
    pub platform: Option<String>,
    pub width: i32,
    pub height: i32,
}

impl Visitor {
    pub fn new(project_id: i64, visitor: &PubVisitor, user_agent: &str) -> Self {
        let mut val = Visitor {
            project: project_id,
            region: TIMEZONES.get(&visitor.tz).cloned().map(ToString::to_string),
            timezone: visitor.tz.clone(),
            language: visitor.lang.clone(),
            width: visitor.screen.0,
            height: visitor.screen.1,
            ..Default::default()
        };

        let ua = UA_PARSER.parse(user_agent);
        let browser = ua.user_agent.family.to_string();
        if !browser.is_empty() {
            val.browser = Some(browser);
        };
        let platform = ua.os.family.to_string();
        if !platform.is_empty() {
            val.platform = Some(platform);
        };

        let mut hasher = Hasher::new();
        hasher.write(val.project as u64);
        if let Some(region) = &val.region {
            hasher.write_bytes(region.as_bytes());
        }
        hasher.write_bytes(val.timezone.as_bytes());
        hasher.write_bytes(val.language.as_bytes());
        if let Some(browser) = &val.browser {
            hasher.write_bytes(browser.as_bytes());
        }
        if let Some(platform) = &val.platform {
            hasher.write_bytes(platform.as_bytes());
        }
        hasher.write(val.width as u64);
        hasher.write(val.height as u64);

        val.id = hasher.finalize() as i64;
        val
    }
}

#[derive(Debug, Default, Clone)]
pub struct Page {
    pub id: i64,
    pub project: i64,
    pub domain: String,
    pub path: String,
}

impl Page {
    /// Returns an error if the url has no valid domain.
    pub fn new(project_id: i64, url: &Url) -> Result<Self, Error> {
        let mut val = Page {
            project: project_id,
            ..Default::default()
        };
        val.domain = url
            .domain()
            .ok_or(Error::Missing("domain".to_string()))?
            .to_string();
        val.path = url.path().to_string();

        let mut hasher = Hasher::new();
        hasher.write(val.project as u64);
        hasher.write_bytes(val.domain.as_bytes());
        hasher.write_bytes(val.path.as_bytes());

        val.id = hasher.finalize() as i64;
        Ok(val)
    }
}

#[derive(Debug, Default, Clone)]
pub struct UtmParam {
    pub id: i64,
    pub project: i64,
    pub campaign: Option<String>,
    pub content: Option<String>,
    pub medium: Option<String>,
    pub source: Option<String>,
    pub term: Option<String>,
}

impl UtmParam {
    pub fn new(project_id: i64, url: &Url) -> Option<Self> {
        let mut val = UtmParam {
            project: project_id,
            ..Default::default()
        };

        let mut found_any = false;

        for (key, value) in url.query_pairs() {
            match &*key {
                "campaign" => {
                    val.campaign = Some(value.to_string());
                    found_any = true;
                }
                "content" => {
                    val.content = Some(value.to_string());
                    found_any = true;
                }
                "medium" => {
                    val.medium = Some(value.to_string());
                    found_any = true;
                }
                "source" => {
                    val.source = Some(value.to_string());
                    found_any = true;
                }
                "term" => {
                    val.term = Some(value.to_string());
                    found_any = true;
                }
                _ => {}
            }
        }

        if found_any {
            let mut hasher = Hasher::new();
            hasher.write(val.project as u64);

            if let Some(campaign) = &val.campaign {
                hasher.write_bytes(campaign.as_bytes());
            }
            if let Some(content) = &val.content {
                hasher.write_bytes(content.as_bytes());
            }
            if let Some(medium) = &val.medium {
                hasher.write_bytes(medium.as_bytes());
            }
            if let Some(source) = &val.source {
                hasher.write_bytes(source.as_bytes());
            }
            if let Some(term) = &val.term {
                hasher.write_bytes(term.as_bytes());
            }

            val.id = hasher.finalize() as i64;
            Some(val)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Referrer {
    pub id: i64,
    pub project: i64,
    pub domain: String,
}

impl Referrer {
    pub fn new(project_id: i64, referrer: Option<&Url>, host: &str) -> Option<Self> {
        let referrer = referrer?.domain()?.to_lowercase();
        if referrer == host.to_lowercase() {
            return None;
        }

        let mut val = Referrer {
            project: project_id,
            domain: referrer,
            ..Default::default()
        };

        let mut hasher = Hasher::new();
        hasher.write(val.project as u64);
        hasher.write_bytes(val.domain.as_bytes());

        val.id = hasher.finalize() as i64;
        Some(val)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Visit {
    pub time: DateTime<Utc>,
    pub project: i64,
    pub session: i64,
    pub visitor: Visitor,
    pub page: Page,
    pub utm_param: Option<UtmParam>,
    pub referrer: Option<Referrer>,
    pub duration: Option<i32>,
    pub distance: Option<f64>,
}

impl Visit {
    pub fn new(
        project_id: i64,
        session: i64,
        visitor: Visitor,
        page: Page,
        utm_param: Option<UtmParam>,
        referrer: Option<Referrer>,
    ) -> Self {
        Visit {
            time: Utc::now(),
            project: project_id,
            session,
            visitor,
            page,
            utm_param,
            referrer,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Event {
    pub time: DateTime<Utc>,
    pub project: i64,
    pub session: i64,
    pub visitor: Visitor,
    pub page: Page,
    pub name: String,
    pub data: Value,
}

impl Event {
    pub fn new(
        project_id: i64,
        session: i64,
        visitor: Visitor,
        page: Page,
        name: String,
        data: Value,
    ) -> Self {
        Event {
            time: Utc::now(),
            project: project_id,
            session,
            visitor,
            page,
            name,
            data,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing {0}")]
    Missing(String),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_timezones_map() {
        let ch = TIMEZONES
            .get("Europe/Zurich")
            .cloned()
            .expect("entry exists");
        assert_eq!(ch, "CH");
    }

    #[test]
    fn smoke_test_ua_parser() {
        let user_agent = "Mozilla/5.0 (Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36";
        let user_agent = UA_PARSER.parse(user_agent);
        assert_eq!(user_agent.user_agent.family.to_string(), "Chrome");
        assert_eq!(user_agent.os.family.to_string(), "Linux");
    }
}

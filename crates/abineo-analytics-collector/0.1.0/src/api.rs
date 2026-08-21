use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::{Error, Event, Page, Referrer, UtmParam, Visit, Visitor};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PubVisitor {
    pub tz: String,
    pub lang: String,
    pub screen: (i32, i32),
}

#[derive(Debug, Deserialize)]
pub struct PubPage {
    pub url: Url,
    #[serde(rename = "ref")]
    pub referrer: Option<Url>,
}

#[derive(Debug, Deserialize)]
pub struct PubVisit {
    pub session: String,
    pub visitor: PubVisitor,
    pub page: PubPage,
}

#[derive(Debug, Deserialize)]
pub struct PubExit {
    pub session: String,
    pub visitor: PubVisitor,
    pub page: PubPage,
    pub dur: i32,
    pub dist: f64,
}

#[derive(Debug, Deserialize)]
pub struct PubEvent {
    pub session: String,
    pub visitor: PubVisitor,
    pub page: PubPage,
    pub name: String,
    pub data: Value,
}

pub async fn handle_visit(
    project_id: i64,
    body: PubVisit,
    user_agent: &str,
) -> Result<Visit, Error> {
    let session: i64 = body.session.parse()?;
    let visitor = Visitor::new(project_id, &body.visitor, user_agent);
    let page = Page::new(project_id, &body.page.url)?;
    let utm_param = UtmParam::new(project_id, &body.page.url);
    let referrer = Referrer::new(project_id, body.page.referrer.as_ref(), &page.domain);

    let visit = Visit::new(project_id, session, visitor, page, utm_param, referrer);

    Ok(visit)
}

pub async fn handle_exit(project_id: i64, body: PubExit, user_agent: &str) -> Result<Visit, Error> {
    let session: i64 = body.session.parse()?;
    let visitor = Visitor::new(project_id, &body.visitor, user_agent);
    let page = Page::new(project_id, &body.page.url)?;
    let utm_param = UtmParam::new(project_id, &body.page.url);
    let referrer = Referrer::new(project_id, body.page.referrer.as_ref(), &page.domain);

    let mut visit = Visit::new(project_id, session, visitor, page, utm_param, referrer);
    visit.duration = Some(body.dur);
    visit.distance = Some(body.dist);

    Ok(visit)
}

pub async fn handle_event(
    project_id: i64,
    body: PubEvent,
    user_agent: &str,
) -> Result<Event, Error> {
    let session: i64 = body.session.parse()?;
    let visitor = Visitor::new(project_id, &body.visitor, user_agent);
    let page = Page::new(project_id, &body.page.url)?;

    let event = Event::new(project_id, session, visitor, page, body.name, body.data);

    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        todo!()
    }
}

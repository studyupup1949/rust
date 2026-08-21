//! # Actix Webfinger
//! A library to aid in resolving and providing webfinger objects with the Actix Web web framework.
//!
//! The main functionality this crate provides is through the `Webfinger::fetch` method for Actix
//! Web-based clients, and the `Resolver` trait for Actix Web-based servers.
//!
//! ### Usage
//! First, add Actix Webfinger as a dependency
//!
//! - [Read the documentation on docs.rs](https://docs.rs/actix-webfinger)
//! - [Find the crate on crates.io](https://crates.io/crates/actix-webfinger)
//! - [Hit me up on Mastodon](https://asonix.dog/@asonix)
//!
//! ```toml
//! [dependencies]
//! actix = "0.10.0-alpha.1"
//! actix-web = "3.0.0-alpha.1"
//! actix-webfinger = "0.3.0-alpha.2"
//! ```
//!
//! Then use it in your application
//!
//! #### Client Example
//! ```rust,ignore
//! use actix_webfinger::Webfinger;
//! use awc::Client;
//! use std::error::Error;
//!
//! #[actix_rt::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let client = Client::default();
//!     let wf = Webfinger::fetch(&client, None, "asonix@asonix.dog", "localhost:8000", false).await?;
//!
//!     println!("asonix's webfinger:\n{:#?}", wf);
//!     Ok(())
//! }
//! ```
//!
//! #### Server Example
//! ```rust,ignore
//! use actix_web::{web::Data, App, HttpServer};
//! use actix_webfinger::{Resolver, Webfinger};
//! use std::{error::Error, future::Future, pin::Pin};
//!
//! #[derive(Clone, Debug)]
//! pub struct MyState {
//!     domain: String,
//! }
//!
//! pub struct MyResolver;
//!
//! impl Resolver for MyResolver {
//!     type State = Data<MyState>;
//!     type Error = actix_web::error::JsonPayloadError;
//!
//!     fn find(
//!         account: &str,
//!         domain: &str,
//!         state: S,
//!     ) -> Pin<Box<dyn Future<Output = Result<Option<Webfinger>, Self::Error>>>> {
//!         let w = if domain == state.domain {
//!             Some(Webfinger::new(&format!("{}@{}", account, domain)))
//!         } else {
//!             None
//!         };
//!
//!         Box::pin(async move { Ok(w) })
//!     }
//! }
//!
//! #[actix_rt::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .data(MyState {
//!                 domain: "asonix.dog".to_owned(),
//!             })
//!             .service(actix_webfinger::resource::<MyResolver>())
//!     })
//!     .bind("127.0.0.1:8000")?
//!     .run()
//!     .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Contributing
//! Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.
//!
//! ### License
//!
//! Copyright © 2020 Riley Trautman
//!
//! Actix Webfinger is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//!
//! Actix Webfinger is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Tokio ZMQ.
//!
//! You should have received a copy of the GNU General Public License along with Actix Webfinger. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
use actix_web::{
    dev::RequestHead,
    error::ResponseError,
    guard::Guard,
    http::Method,
    web::{get, Query},
    FromRequest, HttpResponse, Resource,
};
use awc::Client;
use serde_derive::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

/// A predicate for Actix Web route filters
///
/// This predicate matches GET requests with valid Accept headers. A valid Accept header is any
/// Accept headers that matches a superset of `application/jrd+json`.
///
/// Valid Accept Headers
///  - `application/jrd+json'
///  - `application/json`
///  - `application/*`
///  - `*/*`
///
///  ```rust,ignore
///  use actix_web::App;
///  use actix_webfinger::WebfingerGuard;
///
///  let app = App::new()
///     .resource("/.well-known/webfinger", |r| {
///         r.route()
///             .filter(WebfingerGuard)
///             .with(your_route_handler)
///     })
///     .finish();
///  ```
pub struct WebfingerGuard;

impl Guard for WebfingerGuard {
    fn check(&self, request: &RequestHead) -> bool {
        let valid_accept = if let Some(val) = request.headers().get("Accept") {
            if let Ok(s) = val.to_str() {
                s.split(',').any(|v| {
                    let v = if let Some(index) = v.find(';') {
                        v.split_at(index).0
                    } else {
                        v
                    };
                    let trimmed = v.trim();

                    // The following accept mimes are valid
                    trimmed == "application/jrd+json"
                        || trimmed == "application/json"
                        || trimmed == "application/*"
                        || trimmed == "*/*"
                })
            } else {
                // unparsable accept headers are not valid
                false
            }
        } else {
            // no accept header is valid i guess
            true
        };

        valid_accept && request.method == Method::GET
    }
}

/// A simple way to mount the webfinger service to your Actix Web application
///
/// ```rust,ignore
/// use actix_web::HttpServer;
///
/// HttpServer::new(|| {
///     App::new()
///         .service(actix_webfinger::resource::<MyResolver>())
/// })
/// .bind("127.0.0.1:8000")?
/// .start();
/// ```
pub fn resource<R>() -> Resource
where
    R: Resolver + 'static,
{
    actix_web::web::resource("/.well-known/webfinger")
        .guard(WebfingerGuard)
        .route(get().to(endpoint::<R>))
}

/// A simple way to mount the webfinger service inside the /.well-known scope
///
/// ```rust,ignore
/// use actix_web::{App, HttpServer, web::scope};
/// use actix_webfinger::resource;
///
/// HttpServer::new(|| {
///     App::new()
///         .data(())
///         .service(
///             scope("/")
///                 .service(resource::<MyResolver>())
///         )
/// })
/// .bind("127.0.0.1:8000")
/// .start();
/// ```
pub fn scoped<R>() -> Resource
where
    R: Resolver + 'static,
{
    actix_web::web::resource("/webfinger")
        .guard(WebfingerGuard)
        .route(get().to(endpoint::<R>))
}

/// The error created if the webfinger resource query is malformed
///
/// Resource queries should have a valid `username@instance` format.
///
/// The following resource formats will not produce errors
///  - `acct:asonix@asonix.dog`
///  - `acct:@asonix@asonix.dog`
///  - `asonix@asonix.dog`
///  - `@asonix@asonix.dog`
///
/// The following resource formats will produce errors
///  - `@asonix`
///  - `asonix`
///
/// This error type captures the invalid string for inspection
#[derive(Clone, Debug, thiserror::Error)]
#[error("Resource {0} is invalid")]
pub struct InvalidResource(String);

/// A type representing a valid resource query
///
/// Resource queries should have a valid `username@instance` format.
///
/// The following resource formats will not produce errors
///  - `acct:asonix@asonix.dog`
///  - `acct:@asonix@asonix.dog`
///  - `asonix@asonix.dog`
///  - `@asonix@asonix.dog`
///
/// The following resource formats will produce errors
///  - `@asonix`
///  - `asonix`
///
/// This type implements `FromStr` and `serde::de::Deserialize` so it can be used to enforce valid
/// formatting before the request reaches the route handler.
#[derive(Clone, Debug)]
pub struct WebfingerResource {
    pub scheme: Option<String>,
    pub account: String,
    pub domain: String,
}

impl std::str::FromStr for WebfingerResource {
    type Err = InvalidResource;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (scheme, trimmed) = s
            .find(':')
            .map(|index| {
                let (scheme, trimmed) = s.split_at(index);
                (
                    Some(scheme.to_owned() + ":"),
                    trimmed.trim_start_matches(':'),
                )
            })
            .unwrap_or((None, s));

        let trimmed = trimmed.trim_start_matches('@');

        if let Some(index) = trimmed.find('@') {
            let (account, domain) = trimmed.split_at(index);

            Ok(WebfingerResource {
                scheme,
                account: account.to_owned(),
                domain: domain.trim_start_matches('@').to_owned(),
            })
        } else {
            Err(InvalidResource(s.to_owned()))
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for WebfingerResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<WebfingerResource>()
            .map_err(serde::de::Error::custom)
    }
}

/// A wrapper type for a Webfinger Resource
///
/// This type is used to deserialize from queries like the following:
///  - `resource=acct:asonix@asonix.dog`
///
/// This can be used in Actix Web with the following code:
///
/// ```rust,ignore
/// use actix_web::Query;
/// use actix_webfinger::{WebfingerQuery, WebfingerResource};
///
/// fn my_route(query: Query<WebfingerQuery>) -> String {
///     let WebfingerResource {
///         account,
///         domain,
///     } = query.into_inner().resource;
///
///     // do things
///     String::from("got resource")
/// }
/// ```
#[derive(Clone, Debug, Deserialize)]
pub struct WebfingerQuery {
    resource: WebfingerResource,
}

/// A trait to ease the implementation of Webfinger Resolvers
///
/// ```rust,ignore
/// use actix_webfinger::{Resolver, Webfinger};
/// use std::{future::Future, pin::Pin};
///
/// struct MyResolver;
///
/// impl Resolver for MyResolver {
///     type State = ();
///     type Error = CustomError;
///
///     fn find(
///         account: &str,
///         domain: &str,
///         _state: Self::State,
///     ) -> Pin<Box<dyn Future<Output = Result<Option<Webfinger>, Self::Error>>>> {
///         let webfinger = Webfinger::new(&format!("{}@{}", account, domain));
///
///         // do something
///
///         Box::pin(async move { Ok(Some(webfinger)) })
///     }
/// }
///
/// #[actix_rt::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     HttpServer::new(|| {
///         App::new()
///             .data(())
///             .service(resource::<MyResolver>())
///     })
///     .bind("127.0.0.1:8000")?
///     .run()
///     .await?;
///
///     Ok(())
/// }
/// ```
pub trait Resolver {
    type State: FromRequest + 'static;
    type Error: ResponseError + 'static;

    fn find(
        scheme: Option<&str>,
        account: &str,
        domain: &str,
        state: Self::State,
    ) -> Pin<Box<dyn Future<Output = WebfingerResult<Self::Error>>>>;
}
type WebfingerResult<E> = Result<Option<Webfinger>, E>;

pub fn endpoint<R>(
    (query, state): (Query<WebfingerQuery>, R::State),
) -> Pin<Box<dyn Future<Output = Result<HttpResponse, R::Error>>>>
where
    R: Resolver,
{
    let WebfingerResource {
        scheme,
        account,
        domain,
    } = query.into_inner().resource;

    Box::pin(async move {
        match R::find(scheme.as_deref(), &account, &domain, state).await? {
            Some(w) => Ok(w.respond()),
            None => Ok(HttpResponse::NotFound().finish()),
        }
    })
}

/// The webfinger Link type
///
/// All links have a `rel` the determines what the link is describing, most have a `type` that adds
/// some more context, and a `href` to point to, or contain the data.
///
/// In some cases, the Link can have a `rel` and a `template` and nothing else.
///
/// This type can be serialized and deserialized
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub rel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// renamed to `type` via serde
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// The webfinger type
///
/// Webfinger data has three parts
///  - aliases
///  - links
///  - subject
///
/// `subject` defines the name or primary identifier of the object being looked up.
/// `aliases` are alternate names for the object.
/// `links` are references to more information about the subject, and can be in a variety of
/// formats. For example, some links will reference activitypub person objects, and others will
/// contain public key data.
///
/// This type can be serialized and deserialized
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Webfinger {
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
    pub subject: String,
}

impl Webfinger {
    /// Create a new Webfinger with the given subject
    pub fn new(subject: &str) -> Self {
        Webfinger {
            aliases: Vec::new(),
            links: Vec::new(),
            subject: subject.to_owned(),
        }
    }

    /// Add multiple aliases to the Webfinger
    pub fn add_aliases(&mut self, aliases: &[String]) -> &mut Self {
        self.aliases.extend_from_slice(aliases);
        self
    }

    /// Add a single alias to the Webfinger
    pub fn add_alias(&mut self, alias: &str) -> &mut Self {
        self.aliases.push(alias.to_owned());
        self
    }

    /// Get the aliases for this Webfinger
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// Add multiple Links to this Webfinger
    pub fn add_links(&mut self, links: &[Link]) -> &mut Self {
        self.links.extend_from_slice(links);
        self
    }

    /// Add single Link to this Webfinger
    pub fn add_link(&mut self, link: Link) -> &mut Self {
        self.links.push(link);
        self
    }

    /// Get the Links from this Webfinger
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Add an ActivityPub link to this Webfinger
    ///
    /// Since ActivityPub extends JsonLD, this also adds an ActivityStreams JsonLD link
    pub fn add_activitypub(&mut self, href: &str) -> &mut Self {
        self.links.push(Link {
            rel: "self".to_owned(),
            kind: Some("application/activity+json".to_owned()),
            href: Some(href.to_owned()),
            template: None,
        });
        self.add_json_ld(href, "https://www.w3.org/ns/activitystreams")
    }

    /// Add a JsonLD Link to this Webfinger
    pub fn add_json_ld(&mut self, href: &str, profile: &str) -> &mut Self {
        self.links.push(Link {
            rel: "self".to_owned(),
            kind: Some(format!("application/ld+json; profile=\"{}\"", profile)),
            href: Some(href.to_owned()),
            template: None,
        });
        self
    }

    /// Get an ActivityPub link from this Webfinger
    pub fn activitypub(&self) -> Option<&Link> {
        self.links.iter().find(|l| {
            l.rel == "self"
                && l.kind
                    .as_ref()
                    .map(|k| k == "application/activity+json")
                    .unwrap_or(false)
        })
    }

    /// Get a JsonLD link from this Webfinger
    pub fn json_ld(&self) -> Option<&Link> {
        self.links.iter().find(|l| {
            l.rel == "self"
                && l.kind
                    .as_ref()
                    .map(|k| k.starts_with("application/ld+json"))
                    .unwrap_or(false)
        })
    }

    /// Add a profile link to this Webfinger
    pub fn add_profile(&mut self, href: &str) -> &mut Self {
        self.links.push(Link {
            rel: "http://webfinger.net/rel/profile-page".to_owned(),
            href: Some(href.to_owned()),
            kind: Some("text/html".to_owned()),
            template: None,
        });
        self
    }

    /// Get a profile link from this Webfinger
    pub fn profile(&self) -> Option<&Link> {
        self.links
            .iter()
            .find(|l| l.rel == "http://webfinger.net/rel/profile-page")
    }

    /// Add an atom link to this Webfinger
    pub fn add_atom(&mut self, href: &str) -> &mut Self {
        self.links.push(Link {
            rel: "http://schemas.google.com/g/2010#updates-from".to_owned(),
            href: Some(href.to_owned()),
            kind: Some("application/atom+xml".to_owned()),
            template: None,
        });
        self
    }

    /// Get an atom link from this Webfinger
    pub fn atom(&self) -> Option<&Link> {
        self.links
            .iter()
            .find(|l| l.rel == "http://schemas.google.com/g/2010#updates-from")
    }

    /// Set a salmon link from this Webfinger
    pub fn add_salmon(&mut self, href: &str) -> &mut Self {
        self.links.push(Link {
            rel: "salmon".to_owned(),
            href: Some(href.to_owned()),
            kind: None,
            template: None,
        });
        self
    }

    /// Get a salmon link from this Webfinger
    pub fn salmon(&self) -> Option<&Link> {
        self.links.iter().find(|l| l.rel == "salmon")
    }

    /// Set a magic public key link for this Webfinger
    pub fn add_magic_public_key(&mut self, magic_public_key: &str) -> &mut Self {
        self.links.push(Link {
            rel: "magic-public-key".to_owned(),
            href: Some(format!(
                "data:application/magic-public-key,{}",
                magic_public_key
            )),
            kind: None,
            template: None,
        });
        self
    }

    /// Get a magic public key link from this Webfinger
    pub fn magic_public_key(&self) -> Option<&Link> {
        self.links.iter().find(|l| l.rel == "magic-public-key")
    }

    /// Set an ostatus link for this Webfinger
    pub fn add_ostatus(&mut self, template: &str) -> &mut Self {
        self.links.push(Link {
            rel: "http://ostatus.org/schema/1.0/subscribe".to_owned(),
            href: None,
            kind: None,
            template: Some(template.to_owned()),
        });
        self
    }

    /// Get an ostatus link from this Webfinger
    pub fn ostatus(&self) -> Option<&Link> {
        self.links
            .iter()
            .find(|l| l.rel == "http://ostatus.org/schema/1.0/subscribe")
    }

    /// Turn this Webfinger into an actix web HttpResponse
    pub fn respond(self) -> HttpResponse {
        HttpResponse::Ok()
            .content_type("application/jrd+json")
            .json(&self)
    }

    /// Fetch a webfinger with subject `user` from a given `domain`
    ///
    /// This method takes a `Client` so derivative works can provide their own configured clients
    /// rather this library generating it's own http clients.
    pub async fn fetch(
        client: &Client,
        scheme: Option<&str>,
        user: &str,
        domain: &str,
        https: bool,
    ) -> Result<Self, FetchError> {
        let url = format!(
            "{}://{}/.well-known/webfinger?resource={}{}",
            if https { "https" } else { "http" },
            domain,
            scheme.unwrap_or("acct:"),
            user
        );

        let mut res = client
            .get(url)
            .append_header(("Accept", "application/jrd+json"))
            .send()
            .await
            .map_err(|_| FetchError::Send)?;

        res.json::<Webfinger>().await.map_err(|_| FetchError::Parse)
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Failed to send request")]
    Send,
    #[error("Failed to parse response JSON")]
    Parse,
}

#[cfg(test)]
mod tests {
    use crate::{Webfinger, WebfingerQuery};

    const SIR_BOOPS: &'static str = r#"{"subject":"acct:Sir_Boops@sergal.org","aliases":["https://mastodon.sergal.org/@Sir_Boops","https://mastodon.sergal.org/users/Sir_Boops"],"links":[{"rel":"http://webfinger.net/rel/profile-page","type":"text/html","href":"https://mastodon.sergal.org/@Sir_Boops"},{"rel":"http://schemas.google.com/g/2010#updates-from","type":"application/atom+xml","href":"https://mastodon.sergal.org/users/Sir_Boops.atom"},{"rel":"self","type":"application/activity+json","href":"https://mastodon.sergal.org/users/Sir_Boops"},{"rel":"salmon","href":"https://mastodon.sergal.org/api/salmon/1"},{"rel":"magic-public-key","href":"data:application/magic-public-key,RSA.vwDujxmxoYHs64MyVB3LG5ZyBxV3ufaMRBFu42bkcTpISq1WwZ-3Zb6CI8zOO-nM-Q2llrVRYjZa4ZFnOLvMTq_Kf-Zf5wy2aCRer88gX-MsJOAtItSi412y0a_rKOuFaDYLOLeTkRvmGLgZWbsrZJOp-YWb3zQ5qsIOInkc5BwI172tMsGeFtsnbNApPV4lrmtTGaJ8RiM8MR7XANBOfOHggSt1-eAIKGIsCmINEMzs1mG9D75xKtC_sM8GfbvBclQcBstGkHAEj1VHPW0ch6Bok5_QQppicyb8UA1PAA9bznSFtKlYE4xCH8rlCDSDTBRtdnBWHKcj619Ujz4Qaw==.AQAB"},{"rel":"http://ostatus.org/schema/1.0/subscribe","template":"https://mastodon.sergal.org/authorize_interaction?uri={uri}"}]}"#;

    const QUERIES: [&'static str; 4] = [
        r#"{"resource":"acct:asonix@asonix.dog"}"#,
        r#"{"resource":"asonix@asonix.dog"}"#,
        r#"{"resource":"acct:@asonix@asonix.dog"}"#,
        r#"{"resource":"@asonix@asonix.dog"}"#,
    ];

    #[test]
    fn can_deserialize_sir_boops() {
        let webfinger: Result<Webfinger, _> = serde_json::from_str(SIR_BOOPS);

        assert!(webfinger.is_ok());

        let webfinger = webfinger.unwrap();

        assert!(webfinger.salmon().is_some());
        assert!(webfinger.ostatus().is_some());
        assert!(webfinger.activitypub().is_some());
        assert!(webfinger.atom().is_some());
        assert!(webfinger.magic_public_key().is_some());
        assert!(webfinger.profile().is_some());
    }

    #[test]
    fn can_deserialize_queries() {
        for resource in &QUERIES {
            let res: Result<WebfingerQuery, _> = serde_json::from_str(resource);

            assert!(res.is_ok());

            let query = res.unwrap();

            assert_eq!(query.resource.account, "asonix");
            assert_eq!(query.resource.domain, "asonix.dog");
        }
    }
}

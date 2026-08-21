//! # Actix Webfinger
//! A library to aid in resolving and providing webfinger objects with the Actix Web web framework.
//!
//! The main functionality this crate provides is through the `Webfinger::fetch` method for Actix
//! Web-based clients, and the `Resolver<S>` trait for Actix Web-based servers.
//!
//! ### Usage
//! First, add Actix Webfinger as a dependency
//!
//! ```toml
//! [dependencies]
//! actix = "0.7"
//! actix-web = "0.7"
//! actix-webfinger = "0.1"
//! ```
//!
//! Then use it in your application
//!
//! #### Client Example
//! ```rust,ignore
//! use actix::{Actor, System};
//! use actix_web::client::ClientConnector;
//! use actix_webfinger::Webfinger;
//! use futures::Future;
//! use openssl::ssl::{SslConnector, SslMethod};
//! 
//! fn main() {
//!     let sys = System::new("asonix");
//! 
//!     let ssl_conn = SslConnector::builder(SslMethod::tls()).unwrap().build();
//!     let conn = ClientConnector::with_connector(ssl_conn).start();
//! 
//!     let fut = Webfinger::fetch(conn, "asonix@asonix.dog", "localhost:8000", false)
//!         .map(move |w: Webfinger| {
//!             println!("asonix's webfinger:\n{:#?}", w);
//! 
//!             System::current().stop();
//!         })
//!         .map_err(|e| eprintln!("Error: {}", e));
//! 
//!     actix::spawn(fut);
//! 
//!     let _ = sys.run();
//! }
//! ```
//!
//! #### Server Example
//! ```rust,ignore
//! use actix_web::server;
//! use actix_webfinger::{Resolver, Webfinger};
//! use futures::{future::IntoFuture, Future};
//! 
//! #[derive(Clone, Debug)]
//! pub struct MyState {
//!     domain: String,
//! }
//! 
//! pub struct MyResolver;
//! 
//! impl Resolver<MyState> for MyResolver {
//!     type Error = actix_web::error::JsonPayloadError;
//! 
//!     fn find(
//!         account: &str,
//!         domain: &str,
//!         state: &MyState,
//!     ) -> Box<dyn Future<Item = Option<Webfinger>, Error = Self::Error>> {
//!         let w = if domain == state.domain {
//!             Some(Webfinger::new(&format!("{}@{}", account, domain)))
//!         } else {
//!             None
//!         };
//! 
//!         Box::new(Ok(w).into_future())
//!     }
//! }
//! 
//! fn main() {
//!     server::new(|| {
//!         actix_webfinger::app::<MyState, MyResolver>(MyState {
//!             domain: "asonix.dog".to_owned(),
//!         })
//!     })
//!     .bind("127.0.0.1:8000")
//!     .unwrap()
//!     .run();
//! }
//! ```
//!
//! ### Contributing
//! Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.
//! 
//! ### License
//! 
//! Copyright © 2019 Riley Trautman
//! 
//! Actix Webfinger is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//! 
//! Actix Webfinger is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Tokio ZMQ.
//! 
//! You should have received a copy of the GNU General Public License along with Actix Webfinger. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
use actix::Addr;
use actix_web::{
    client::{self, ClientConnector},
    error::ResponseError,
    http::Method,
    pred::Predicate,
    App, HttpMessage, HttpResponse, Query, Request, State,
};
use failure::Fail;
use futures::{future::IntoFuture, Future};
use serde_derive::{Deserialize, Serialize};

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
///  use actix_webfinger::WebfingerPredicate;
///
///  let app = App::new()
///     .resource("/.well-known/webfinger", |r| {
///         r.route()
///             .filter(WebfingerPredicate)
///             .with(your_route_handler)
///     })
///     .finish();
///  ```
pub struct WebfingerPredicate;

impl<S> Predicate<S> for WebfingerPredicate {
    fn check(&self, req: &Request, _: &S) -> bool {
        if let Some(val) = req.headers().get("Accept") {
            if let Ok(s) = val.to_str() {
                return s.split(",").any(|v| {
                    let v = if let Some(index) = v.find(';') {
                        v.split_at(index).0
                    } else {
                        v
                    };
                    let trimmed = v.trim();

                    trimmed == "application/jrd+json"
                        || trimmed == "application/json"
                        || trimmed == "application/*"
                        || trimmed == "*/*"
                }) && req.method() == Method::GET;
            }
        }

        false
    }
}

/// A simple way to mount the webfinger app to your Actix Web application
///
/// ```rust,ignore
/// use actix_web::server;
/// use actix_webfinger::app;
///
/// server::new(|| {
///     app::<_, MyResolver>(my_state)
/// })
/// .bind("127.0.0.1:8000")?
/// .run();
/// ```
pub fn app<S, R>(state: S) -> App<S>
where
    R: Resolver<S> + 'static,
    S: 'static,
{
    App::with_state(state).resource("/.well-known/webfinger", |r| {
        r.route().filter(WebfingerPredicate).with(R::endpoint)
    })
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
#[derive(Clone, Debug, Fail)]
#[fail(display = "Resource {} is invalid", _0)]
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
    pub account: String,
    pub domain: String,
}

impl std::str::FromStr for WebfingerResource {
    type Err = InvalidResource;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim_start_matches("acct:").trim_start_matches('@');

        if let Some(index) = trimmed.find('@') {
            let (account, domain) = trimmed.split_at(index);

            Ok(WebfingerResource {
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
/// This trait can be implemented, and the provided `endpoint` method used to serve requests to an
/// Actix Web server.
///
/// ```rust,ignore
/// use actix_webfinger::{Resolver, Webfinger};
/// use futures::{Future, future::IntoFuture};
///
/// struct MyResolver;
///
/// impl<S> Resolver<S> for MyResolver {
///     type Error = CustomError;
///
///     fn find(
///         account: &str,
///         domain: &str,
///         _state: S
///     ) -> Box<dyn Future<Item = Option<Webfinger>, Error = Self::Error>> {
///         let webfinger = Webfinger::new(&format!("{}@{}", account, domain));
///
///         // do something
///
///         let f = Ok(Some(webfinger)).into_future();
///
///         Box::new(f)
///     }
/// }
///
/// fn main() {
///     server::new(|| {
///         App::new()
///             .resource("/.well-known/webfinger", |r| {
///                 r.with(<MyResolver as Resolver<()>>::endpoint)
///             })
///     })
///     .bind("127.0.0.1:8000")?
///     .run();
/// }
/// ```
pub trait Resolver<S> {
    type Error: ResponseError;

    fn find(
        account: &str,
        domain: &str,
        state: &S,
    ) -> Box<dyn Future<Item = Option<Webfinger>, Error = Self::Error>>;

    fn endpoint(
        (query, state): (Query<WebfingerQuery>, State<S>),
    ) -> Box<dyn Future<Item = HttpResponse, Error = Self::Error>> {
        let WebfingerResource { account, domain } = query.into_inner().resource;

        Box::new(Self::find(&account, &domain, &state).map(|w| match w {
            Some(w) => w.respond(),
            None => HttpResponse::NotFound().finish(),
        }))
    }
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

    /// Add an Activitypub link to this Webfinger
    pub fn add_activitypub(&mut self, href: &str) -> &mut Self {
        self.links.push(Link {
            rel: "self".to_owned(),
            kind: Some("application/activity+json".to_owned()),
            href: Some(href.to_owned()),
            template: None,
        });
        self
    }

    /// Get an Activitypub link from this Webfinger
    pub fn activitypub(&self) -> Option<&Link> {
        self.links.iter().find(|l| {
            l.rel == "self"
                && l.kind
                    .as_ref()
                    .map(|k| k == "application/activity+json")
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
    pub fn set_salmon(&mut self, href: &str) -> &mut Self {
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
    pub fn set_magic_public_key(&mut self, magic_public_key: &str) -> &mut Self {
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
    pub fn set_ostatus(&mut self, template: &str) -> &mut Self {
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
            .json(self)
    }

    /// Fetch a webfinger with subject `user` from a given `domain`
    ///
    /// This method takes an `Addr<ClientConnector>` so derivative works can provide their own SSL
    /// connector implemenation (currently with OpenSSL or Rustls)
    pub fn fetch(
        conn: Addr<ClientConnector>,
        user: &str,
        domain: &str,
        https: bool,
    ) -> Box<dyn Future<Item = Self, Error = actix_web::Error>> {
        let url = format!(
            "{}://{}/.well-known/webfinger?resource=acct:{}",
            if https { "https" } else { "http" },
            domain,
            user
        );

        let fut = client::get(url)
            .with_connector(conn)
            .header("Accept", "application/jrd+json")
            .finish()
            .into_future()
            .and_then(|r| {
                r.send()
                    .from_err()
                    .and_then(|res| res.json::<Webfinger>().from_err())
            });

        Box::new(fut)
    }
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

#![deny(missing_docs)]

//! # Actix Sled Session
//! _An Actix Web Session Backend using the Sled embedded database_
//!
//! This session backend sets an cookie with a unique ID for each incoming request that doesn't
//! already have an ID set, and uses that ID as a key to look up session data from a Sled tree.
//!
//! ### Usage
//! #### Update your Cargo.toml
//! ```toml
//! [dependencies]
//! actix = "0.8"
//! actix-sled-session = "0.1"
//! actix-web = "1.0"
//! ```
//!
//! #### Use it in your project
//! ```rust
//! use actix::System;
//! use actix_web::{web, App, HttpServer};
//! use actix_sled_session::{Session, SledSession};
//!
//! fn index(session: Session) -> String {
//!     if let Ok(Some(item)) = session.get::<usize>("item") {
//!         println!("item, {}", item);
//!         session.clear();
//!         return format!("Got item, {}", item);
//!     }
//!
//!     let _ = session.set::<usize>("item", 3);
//!     String::from("Set item!")
//! }
//!
//! # fn main() -> Result<(), failure::Error> {
//! let sys = System::new("example");
//! let session_backend = SledSession::new_default()?;
//!
//! HttpServer::new(move || {
//!     App::new()
//!         .wrap(session_backend.clone())
//!         .route("/", web::get().to(index))
//! })
//!     .bind("127.0.0.1:9876")?
//!     .start();
//!
//! // Commented to prevent never-ending tests
//! // sys.run()?;
//! # Ok(())
//! # }
//! ```

use actix::{Actor, Addr};
use actix_session::SessionStatus;
use actix_sled_cache::bincode::Cache;
use actix_web::{
    cookie::{Cookie, CookieJar, Key},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::BlockingError,
    http::header::{HeaderValue, SET_COOKIE},
    web, HttpMessage, ResponseError,
};
use chrono::Duration;
use failure::Fail;
use futures::{
    future::{ok, Future, FutureResult},
    Poll,
};
use log::error;
use rand::RngCore;
use sled_extensions::{bincode::expiring::Tree, Config, Db};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};
use uuid::Uuid;

pub use actix_session::Session;

#[derive(Clone)]
/// The session middleware
///
/// This struct must be passed into a `.wrap` method when constructing an Actix Web App
pub struct SledSession {
    name: String,
    key: Key,
    secure: bool,
    db: Db,
    tree: Tree<HashMap<String, String>>,
    cache: Addr<Cache<HashMap<String, String>>>,
}

#[doc(hidden)]
pub struct SledSessionMiddleware<S> {
    service: Rc<RefCell<S>>,
    session: SledSession,
}

#[derive(Debug, Fail)]
#[doc(hidden)]
pub enum SledSessionError {
    #[fail(display = "Error in sled, {}", _0)]
    Session(#[cause] sled_extensions::Error),

    #[fail(display = "Error doing cookie stuff")]
    Cookie,

    #[fail(display = "Operation canceled")]
    Canceled,
}

impl SledSession {
    /// Create a new middleware with the provided sled database
    ///
    /// The provided name is used to open the session tree and set the session cookie
    /// The key is used to encrypt the session cookie
    /// The secure flag determines whether the cookie must be sent over HTTPS
    /// Expiry sets how long the items in the session cache are valid for
    pub fn new(
        db: Db,
        name: &str,
        secure: bool,
        key: &[u8],
        expiry: Option<Duration>,
    ) -> Result<Self, actix_sled_cache::Error> {
        let mut cache_builder = Cache::builder(db.clone(), name);

        if let Some(expiry) = expiry {
            if let Ok(expiry) = expiry.to_std() {
                cache_builder.frequency(expiry);
            }
            cache_builder.as_mut().expiration_length(expiry);
        }

        cache_builder.as_mut().extend_on_update().extend_on_fetch();

        let cache = cache_builder.build()?;
        let tree = cache.tree();
        let cache = cache.start();

        Ok(SledSession {
            name: name.to_owned(),
            key: Key::from_master(key),
            secure,
            db,
            tree,
            cache,
        })
    }

    /// Create a new default middleware backed by a temporary, compressed sled database
    ///
    /// The defaultlt expiry is set to 7 days, and the frequency check is also set to 7 days. This
    /// means session data can be valid betwen 7 and 14 days.
    /// the Secure flag defaults to false
    /// The cookie's key is generated randomly
    pub fn new_default() -> Result<Self, actix_sled_cache::Error> {
        let db = Config::default()
            .use_compression(true)
            .compression_factor(10)
            .temporary(true)
            .open()
            .map_err(sled_extensions::Error::from)?;

        let expiry = Duration::days(7);

        let mut key: [u8; 32] = [0; 32];

        rand::thread_rng().fill_bytes(&mut key);

        Self::new(db, "actix-sled-session-storage", false, &key, Some(expiry))
    }

    fn fetch_identity(&self, req: &ServiceRequest) -> (bool, Uuid) {
        if let Some(cookie) = req.cookie(&self.name) {
            let mut jar = CookieJar::new();
            jar.add_original(cookie.clone());

            if let Some(cookie) = jar.private(&self.key).get(&self.name) {
                if let Ok(uuid) = serde_json::from_str(cookie.value()) {
                    return (false, uuid);
                }
            }
        }

        (true, Uuid::new_v4())
    }

    fn set_identity<B>(
        &self,
        is_new: bool,
        status: SessionStatus,
        key: Uuid,
        res: &mut ServiceResponse<B>,
    ) -> Result<(), SledSessionError> {
        let value = match status {
            SessionStatus::Changed => {
                serde_json::to_string(&key).map_err(|_| SledSessionError::Cookie)?
            }
            SessionStatus::Unchanged => {
                if is_new {
                    serde_json::to_string(&key).map_err(|_| SledSessionError::Cookie)?
                } else {
                    return Ok(());
                }
            }
            SessionStatus::Purged => {
                self.remove_identity(res)?;
                return Ok(());
            }
            SessionStatus::Renewed => {
                serde_json::to_string(&key).map_err(|_| SledSessionError::Cookie)?
            }
        };

        let mut cookie = Cookie::new(self.name.clone(), value);
        cookie.set_path("/".to_owned());
        cookie.set_secure(self.secure);
        cookie.set_http_only(true);

        let mut jar = CookieJar::new();

        jar.private(&self.key).add(cookie);

        for cookie in jar.delta() {
            let val = HeaderValue::from_str(&cookie.encoded().to_string())
                .map_err(|_| SledSessionError::Cookie)?;
            res.headers_mut().append(SET_COOKIE, val);
        }

        Ok(())
    }

    fn remove_identity<B>(&self, res: &mut ServiceResponse<B>) -> Result<(), SledSessionError> {
        let mut cookie = Cookie::named(self.name.clone());
        cookie.set_value("");
        cookie.set_max_age(time::Duration::seconds(0));
        cookie.set_expires(time::now() - time::Duration::days(365));

        let val =
            HeaderValue::from_str(&cookie.to_string()).map_err(|_| SledSessionError::Cookie)?;
        res.headers_mut().append(SET_COOKIE, val);

        Ok(())
    }

    fn fetch_session(
        &self,
        key: Uuid,
    ) -> impl Future<Item = HashMap<String, String>, Error = SledSessionError> {
        let tree = self.tree.cloned();

        web::block(move || {
            tree.get(key.to_string())
                .map(|opt| opt.unwrap_or(HashMap::new()))
        })
        .map_err(|e| {
            error!("{}", e);
            e.into()
        })
    }

    fn store_session(
        &self,
        key: Uuid,
        status: SessionStatus,
        hm: HashMap<String, String>,
    ) -> impl Future<Item = (), Error = SledSessionError> {
        let tree = self.tree.cloned();

        web::block(move || match status {
            SessionStatus::Changed => tree.insert(key.to_string().as_bytes(), hm).map(|_| ()),
            SessionStatus::Purged => tree.remove(key.to_string()).map(|_| ()),
            SessionStatus::Renewed => Ok(()),
            SessionStatus::Unchanged => Ok(()),
        })
        .map_err(|e| {
            error!("{}", e);
            e.into()
        })
    }
}

impl<S, B: 'static> Transform<S> for SledSession
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>
        + 'static,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = SledSessionMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SledSessionMiddleware {
            service: Rc::new(RefCell::new(service)),
            session: self.clone(),
        })
    }
}

impl<S, B: 'static> Service for SledSessionMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>
        + 'static,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.borrow_mut().poll_ready()
    }

    fn call(&mut self, mut req: Self::Request) -> Self::Future {
        let service = self.service.clone();
        let session = self.session.clone();
        let (is_new, uuid) = session.fetch_identity(&req);

        Box::new(
            session
                .fetch_session(uuid)
                .from_err()
                .and_then(move |hashmap| {
                    Session::set_session(hashmap.into_iter(), &mut req);
                    service.borrow_mut().call(req)
                })
                .and_then(move |mut res| {
                    let (status, changes) = Session::get_changes(&mut res);
                    let hm = changes.map(|i| i.collect()).unwrap_or(HashMap::new());

                    session
                        .store_session(uuid, status.clone(), hm)
                        .and_then(move |()| {
                            session.set_identity(is_new, status, uuid, &mut res)?;
                            Ok(res)
                        })
                        .from_err()
                }),
        )
    }
}

impl From<sled_extensions::Error> for SledSessionError {
    fn from(e: sled_extensions::Error) -> Self {
        SledSessionError::Session(e)
    }
}

impl<E> From<BlockingError<E>> for SledSessionError
where
    E: Into<Self> + Debug,
{
    fn from(e: BlockingError<E>) -> Self {
        match e {
            BlockingError::Error(e) => e.into(),
            BlockingError::Canceled => SledSessionError::Canceled,
        }
    }
}

impl ResponseError for SledSessionError {
    // default to InternalServerError
}

//! # Actix Webpack Proxy
//! _A simple way to proxy web requests to a webpack dev server with Actix_
//!
//! ### Usage
//! *This is intended to only be used in Development. For production cases, you should be serving
//! static assets.*
//!
//! First, add this project to your dependencies
//! ```toml
//! # Cargo.toml
//! actix-webpack-proxy = "0.1"
//! ```
//!
//! Then, use it in your project:
//! ```rust,ignore
//! // src/main.rs
//! use actix::System;
//! use actix_web::{client::Client, App, HttpServer};
//! use actix_webpack_proxy::{default_route, ws_resource, DefaultProxy};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let sys = System::new("dev-system");
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .data(Client::new())
//!             .data(DefaultProxy)
//!             .service(ws_resource::<DefaultProxy>())
//!             .default_service(default_route::<DefaultProxy>())
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .start();
//!
//!     sys.run()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Contributing
//! Unless otherwise stated, all contributions to this project will be licensed under the CSL with
//! the exceptions listed in the License section of this file.
//!
//! ### License
//! This work is licensed under the Cooperative Software License. This is not a Free Software
//! License, but may be considered a "source-available License." For most hobbyists, self-employed
//! developers, worker-owned companies, and cooperatives, this software can be used in most
//! projects so long as this software is distributed under the terms of the CSL. For more
//! information, see the provided LICENSE file. If none exists, the license can be found online
//! [here](https://lynnesbian.space/csl/). If you are a free software project and wish to use this
//! software under the terms of the GNU Affero General Public License, please contact me at
//! [asonix@asonix.dog](mailto:asonix@asonix.dog) and we can sort that out. If you wish to use this
//! project under any other license, especially in proprietary software, the answer is likely no.

use actix::{io::SinkWrite, Actor, ActorContext, AsyncContext, StreamHandler};
use actix_codec::{AsyncRead, AsyncWrite, Framed};
use actix_web::{client::Client, web, Error, HttpRequest, HttpResponse, ResponseError};
use actix_web_actors::ws;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame},
};
use failure::Fail;
use futures::{
    stream::{SplitSink, SplitStream, Stream},
    Future,
};
use log::{error, info};

/// The default route for your web application
///
/// Any web request that doesn't match a route you have defined should be proxied to the webpack
/// dev server.
pub fn default_route<P>() -> actix_web::Route
where
    P: Proxy + 'static,
{
    web::route().to_async(web_proxy::<P>)
}

/// The websocket resource for hot-reloading
///
/// Webpack's dev server uses websockets in order to notify the web-app of new code and peform a
/// hot-reload. This resource proxies requests to the webpack websocket endpoint.
pub fn ws_resource<P>() -> actix_web::Resource
where
    P: Proxy + 'static,
{
    web::resource("/sockjs-node/{path1}/{path2}/websocket")
        .default_service(web::route().to_async(ws_proxy::<P>))
}

/// Define a trait to enable custom setups
///
/// In most cases, the [`DefaultProxy`]() type should be enough to set up a working environment,
/// but sometimes your webpack server will run on a custom port, or maybe you'll want to source the
/// webpack address from the environment. In these cases, a custom `Proxy` can be implemented.
pub trait Proxy {
    /// Returns the address of the webpack dev server
    ///
    /// When using create-react-app or create-elm-app, the webpack dev server is configured to run
    /// on `localhost:3000` and the [`DefaultProxy`]() type can be used.
    fn host(&self) -> &str;
}

/// The default Proxy implementation
///
/// This proxy will return `localhost:3000` from the `host` method.
pub struct DefaultProxy;

impl Proxy for DefaultProxy {
    fn host(&self) -> &str {
        "localhost:3000"
    }
}

fn web_proxy<P: Proxy>(
    (req, client, proxy, payload): (HttpRequest, web::Data<Client>, web::Data<P>, web::Payload),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let creq = client
        .request_from(
            format!(
                "http://{}/{}",
                proxy.host(),
                req.path().trim_start_matches('/')
            ),
            req.head(),
        )
        .no_decompress();

    let creq = if let Some(addr) = req.head().peer_addr {
        creq.header("X-Forwarded-For", format!("{}", addr.ip()))
    } else {
        creq
    };

    creq.send_stream(payload)
        .map_err(|e| {
            error!("Error: {}", e);
            Error::from(e)
        })
        .and_then(|res| {
            let mut client_resp = HttpResponse::build(res.status());
            for (header_name, header_value) in res
                .headers()
                .iter()
                .filter(|(h, _)| *h != "connection" && *h != "content-length")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }
            client_resp.streaming(res)
        })
}

fn ws_proxy<P: Proxy + 'static>(
    (req, client, proxy, payload): (HttpRequest, web::Data<Client>, web::Data<P>, web::Payload),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let creq = client.ws(format!(
        "http://{}/{}?{}",
        proxy.host(),
        req.path().trim_start_matches('/'),
        req.query_string(),
    ));

    let creq = req.headers().iter().fold(creq, |creq, (k, v)| {
        if k == "cookie" {
            creq
        } else {
            creq.set_header(k.to_owned(), v.to_owned())
        }
    });

    creq.connect()
        .map_err(|e| {
            error!("WS Error: {}", e);
            MyError.into()
        })
        .and_then(move |(_, framed)| {
            ws::handshake(&req)
                .map(move |res| (res, framed))
                .map_err(|e| {
                    error!("WS Error: {}", e);
                    MyError.into()
                })
        })
        .and_then(|(mut res, framed)| {
            let (sink, server_stream) = framed.split();

            res.streaming(ws::WebsocketContext::create(
                ClientActor(Some(sink), None, Some(server_stream)),
                payload,
            ))
        })
}

#[derive(Clone, Debug, Fail)]
#[fail(display = "Websocket Error")]
struct MyError;

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}

struct ClientActor<T>(
    Option<SplitSink<Framed<T, Codec>>>,
    Option<SinkWrite<SplitSink<Framed<T, Codec>>>>,
    Option<SplitStream<Framed<T, Codec>>>,
)
where
    T: AsyncRead + AsyncWrite;

impl<T> Actor for ClientActor<T>
where
    T: AsyncRead + AsyncWrite + 'static,
{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let stream = self.2.take().unwrap();
        let sink = self.0.take().unwrap();
        self.1 = Some(SinkWrite::new(sink, ctx));
        ctx.add_stream(stream);
    }
}

impl<T> StreamHandler<ws::Message, ws::ProtocolError> for ClientActor<T>
where
    T: AsyncRead + AsyncWrite + 'static,
{
    fn handle(&mut self, msg: ws::Message, _: &mut Self::Context) {
        self.1.as_mut().map(|s| s.write(msg).unwrap());
    }
}

impl<T> StreamHandler<Frame, WsProtocolError> for ClientActor<T>
where
    T: AsyncRead + AsyncWrite + 'static,
{
    fn handle(&mut self, msg: Frame, ctx: &mut Self::Context) {
        match msg {
            Frame::Text(Some(text)) => {
                ctx.text(String::from_utf8((&text.freeze()).to_vec()).unwrap());
            }
            Frame::Text(None) => {
                ctx.text(String::from(""));
            }
            Frame::Binary(Some(bin)) => {
                ctx.binary(bin);
            }
            Frame::Binary(None) => {
                ctx.binary(Vec::new());
            }
            Frame::Ping(msg) => {
                ctx.ping(&msg);
            }
            Frame::Pong(msg) => {
                ctx.pong(&msg);
            }
            Frame::Close(reason) => {
                ctx.close(reason);
            }
        }
    }

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Connected");
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        info!("Server disconnected");
        ctx.stop()
    }
}

impl<T> actix::io::WriteHandler<WsProtocolError> for ClientActor<T> where
    T: AsyncRead + AsyncWrite + 'static
{
}

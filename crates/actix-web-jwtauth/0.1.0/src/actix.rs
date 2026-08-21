use crate::Jwt;
use crate::JwtDecode;

use futures_util::future;
use serde::de::DeserializeOwned;

use actix_web::{dev, http, web, FromRequest, HttpRequest, HttpResponse};

impl<T: DeserializeOwned + 'static> FromRequest for Jwt<T> {
    type Error = HttpResponse;
    type Config = ();

    type Future = future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        future::ready(parse_token(req).and_then(|token| {
            parse_decoder(req)
                .decode(token)
                .map_err(|err| HttpResponse::Unauthorized().body(err.to_string()))
        }))
    }
}

fn parse_token(req: &HttpRequest) -> Result<&'_ str, HttpResponse> {
    const MISSING_HEADER: &str = "Missing authorization header";
    const INVALID_HEADER: &str = "Invalid authorization header";
    const INVALID_SCHEMA: &str = "Only accept `Bearer <token>` schema";

    req.headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(MISSING_HEADER)
        .and_then(|it| it.to_str().map_err(|_| INVALID_HEADER))
        .and_then(|it| it.strip_prefix("Bearer ").ok_or(INVALID_SCHEMA))
        .map_err(|err| HttpResponse::Unauthorized().body(err))
}

fn parse_decoder(req: &HttpRequest) -> &JwtDecode {
    req.app_data::<JwtDecode>()
        .or_else(|| req.app_data::<web::Data<JwtDecode>>().map(|it| it.as_ref()))
        .expect("Use `app.app_data` or `app.data` to set `JwtDecode`")
}

//! Utils to deal with streams data types.

use crate::result::{AppError, Result};

use actix_http::error::PayloadError;
use actix_web::web::Bytes;
use awc::ResponseBody;
use futures_core::stream::Stream;

/// Read body from an HTTP response as string.
/// The content has to be encoded in UTF-8, otherwise
/// [`AppError::Unexpected`] is returned.
/// # Example
/// ```
/// use actix_contrib_rest::result::Result;
/// use actix_contrib_rest::stream::read_body;
/// use awc::Client;
/// use log::error;
///
/// async fn get_example() -> Result<String> {
///     let client = Client::default();
///     let mut res = client.get("http://example.com/")
///                 .send()
///                 .await
///                 .unwrap_or_else(|e| {
///                     error!("{}", e);
///                     std::process::exit(1);
///                 });
///     read_body(res.body()).await
/// }
/// ```
pub async fn read_body<S>(body: ResponseBody<S>) -> Result<String>
where
    S: Stream<Item = core::result::Result<Bytes, PayloadError>>,
{
    let bytes = body.await.unwrap().to_vec();
    String::from_utf8(bytes).map_err(|e| AppError::Unexpected(e.into()))
}

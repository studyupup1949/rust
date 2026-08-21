//! Actix web 1.x async/await shim.

use futures::{self, compat::Compat, FutureExt, TryFutureExt};
use std::pin::Pin;

/// Convert a async fn into a actix-web handler.
///
/// ```rust
/// use actix_web::{web, App, HttpResponse, Error};
/// use std::time::{Instant, Duration};
/// use tokio::timer::Delay;
/// use actix_web_async_compat::async_compat;
///
/// async fn index() -> Result<HttpResponse, Error> {
///     // wait 2s
///     Delay::new(Instant::now() + Duration::from_secs(2)).await?;
///     ok(HttpResponse::Ok().finish())
/// }
///
/// App::new().service(web::resource("/").route(
///     web::to_async(async_compat(index)))
/// );
/// ```
pub fn async_compat<F, P, R, O, E>(
    f: F,
) -> impl Fn(P) -> Compat<Pin<Box<dyn futures::Future<Output = Result<O, E>>>>> + Clone
where
    F: Fn(P) -> R + Clone + 'static,
    P: 'static,
    R: futures::Future<Output = Result<O, E>> + 'static,
    O: 'static,
    E: 'static,
{
    move |u| f(u).boxed_local().compat()
}

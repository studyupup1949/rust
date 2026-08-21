//! `extract!` macro for defining custom extractors.

/// Define a custom extractor that wraps an existing Actix extractor.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// pub struct User { name: String }
///
/// extract!(UserJson = User, Json);
/// ```
#[macro_export]
macro_rules! extract {
    ($name:ident = $inner:ty, $wrapper:ident) => {
        pub struct $name(pub $inner);

        impl std::ops::Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &$inner {
                &self.0
            }
        }

        impl actix_web::FromRequest for $name
        where
            $inner: serde::de::DeserializeOwned + 'static,
        {
            type Error = actix_web::Error;
            type Future =
                std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

            fn from_request(
                req: &actix_web::HttpRequest,
                payload: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                use actix_web::FromRequest;
                let req = req.clone();
                let mut payload = std::mem::replace(payload, actix_web::dev::Payload::None);
                Box::pin(async move {
                    let inner =
                        actix_web::web::$wrapper::<$inner>::from_request(&req, &mut payload)
                            .await?;
                    Ok($name(inner.into_inner()))
                })
            }
        }
    };
}

pub use extract;

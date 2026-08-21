pub use http;

pub use actix_web_rest_macros as macros;
pub use macros::*;

pub trait RestError:
    std::error::Error + AsRef<str> + serde::ser::Serialize + for<'a> utoipa::ToSchema<'a>
{
    fn status_code(&self) -> http::StatusCode;
}

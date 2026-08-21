use serde::Deserialize;
use utoipa::ToSchema;

/// only used as a type for utoipa. Axum uses multipart as a type/struct
#[derive(Debug, Deserialize, ToSchema)]
pub struct PutThumbnail {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

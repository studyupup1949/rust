//! Error declaration.
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

#[derive(Debug)]
pub enum Error {
    Validate(validator::ValidationErrors),
    Deserialize(DeserializeErrors),
    JsonPayloadError(actix_web::error::JsonPayloadError),
    UrlEncodedError(actix_web::error::UrlencodedError),
    QsError(serde_qs::Error),
}

#[derive(Debug)]
pub enum DeserializeErrors {
    DeserializeQuery(serde_urlencoded::de::Error),
    DeserializeJson(serde_json::error::Error),
    DeserializePath(serde::de::value::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (status, error) = match self {
            Self::Validate(e) => {
                let mut fields: Vec<serde_json::Value> = vec![];

                for (field, err) in e.field_errors() {
                    let errors: Vec<serde_json::Value> = err
                        .iter()
                        .map(|e| {
                            json!({
                                "code": e.code,
                                "msg": e.message
                            })
                        })
                        .collect();

                    fields.push(json!({
                        "field": field,
                        "errors": errors
                    }));
                }

                return write! {f,"{}",
                    json!({
                        "error": {
                            "code": StatusCode::BAD_REQUEST.as_u16(),
                            "status": "VALIDATION_ERROR",
                            "fields": fields
                        }
                    })
                };
            }

            Self::Deserialize(e) => {
                return write! {f,"{}",
                    json!({
                        "error": {
                            "code": StatusCode::BAD_REQUEST.as_u16(),
                            "status": "DESERIALIZE_ERROR",
                            "error": e.to_string()
                        }
                    })
                };
            }

            Self::JsonPayloadError(e) => ("PAYLOAD_ERROR", e.to_string()),
            Self::UrlEncodedError(e) => ("URL_ENCODED_ERROR", e.to_string()),
            Self::QsError(e) => ("QUERY_ERROR", e.to_string()),
        };

        write! {f,"{}",
            json!({
                "error": {
                    "code": StatusCode::BAD_REQUEST.as_u16(),
                    "status": status,
                    "msg": error
                }
            })
        }
    }
}

impl std::fmt::Display for DeserializeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (status, error) = match self {
            Self::DeserializeQuery(e) => ("QUERY_DESERIALIZE_ERROR", e.to_string()),
            Self::DeserializeJson(e) => ("JSON_DESERIALIZE_ERROR", e.to_string()),
            Self::DeserializePath(e) => ("PATH_DESERIALIZE_ERROR", e.to_string()),
        };

        write! {f,"{}",
            json!({
                "error": {
                    "code": StatusCode::BAD_REQUEST.as_u16(),
                    "status": status,
                    "msg": error
                }
            })
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Error::Deserialize(DeserializeErrors::DeserializeJson(error))
    }
}

impl From<serde_qs::Error> for Error {
    fn from(error: serde_qs::Error) -> Self {
        Error::QsError(error)
    }
}

impl From<serde_urlencoded::de::Error> for Error {
    fn from(error: serde_urlencoded::de::Error) -> Self {
        Error::Deserialize(DeserializeErrors::DeserializeQuery(error))
    }
}

impl From<actix_web::error::JsonPayloadError> for Error {
    fn from(error: actix_web::error::JsonPayloadError) -> Self {
        Error::JsonPayloadError(error)
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(error: validator::ValidationErrors) -> Self {
        Error::Validate(error)
    }
}

impl From<actix_web::error::UrlencodedError> for Error {
    fn from(error: actix_web::error::UrlencodedError) -> Self {
        Error::UrlEncodedError(error)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type("application/json")
            .body(self.to_string())
    }
}

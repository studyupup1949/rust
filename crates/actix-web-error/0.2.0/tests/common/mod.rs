use actix_web::{body::MessageBody, http::StatusCode, ResponseError};

pub fn expect_response(error: &impl ResponseError, status: StatusCode, body: &str) {
    assert_eq!(error.status_code(), status);
    assert_eq!(
        error.error_response().into_body().try_into_bytes().unwrap(),
        body
    );
}

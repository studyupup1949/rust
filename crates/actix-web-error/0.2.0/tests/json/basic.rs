use crate::common::expect_response;
use actix_web::http;

#[derive(Debug, thiserror::Error, actix_web_error::Json)]
#[error("Error: {0}")]
#[status(400)]
struct MyError(&'static str);

#[derive(Debug, thiserror::Error, actix_web_error::Json)]
#[error("Error: {0}")]
#[status(BAD_REQUEST)]
struct MyError2(&'static str);

#[derive(Debug, thiserror::Error, actix_web_error::Json)]
#[status(400)]
enum MyEnum {
    #[error("a")]
    BadRequest,
    #[error("b")]
    AnotherBadRequest,
    #[error("c")]
    #[status(500)]
    Internal,
}

#[derive(Debug, thiserror::Error, actix_web_error::Json)]
enum MyEnum2 {
    #[error("a")]
    #[status(BAD_REQUEST)]
    BadRequest,
    #[error("b")]
    #[status(400)]
    AnotherBadRequest,
    #[error("c")]
    #[status(500)]
    Internal,
}
#[test]
fn basic() {
    use http::StatusCode;

    expect_response(
        &MyError("xd"),
        StatusCode::BAD_REQUEST,
        r#"{"error":"Error: xd"}"#,
    );
    expect_response(
        &MyError2("xd"),
        StatusCode::BAD_REQUEST,
        r#"{"error":"Error: xd"}"#,
    );
}

#[test]
fn basic_enum() {
    use http::StatusCode;

    expect_response(
        &MyEnum::BadRequest,
        StatusCode::BAD_REQUEST,
        r#"{"error":"a"}"#,
    );
    expect_response(
        &MyEnum::AnotherBadRequest,
        StatusCode::BAD_REQUEST,
        r#"{"error":"b"}"#,
    );
    expect_response(
        &MyEnum::Internal,
        StatusCode::INTERNAL_SERVER_ERROR,
        r#"{"error":"c"}"#,
    );
    expect_response(
        &MyEnum2::BadRequest,
        StatusCode::BAD_REQUEST,
        r#"{"error":"a"}"#,
    );
    expect_response(
        &MyEnum2::AnotherBadRequest,
        StatusCode::BAD_REQUEST,
        r#"{"error":"b"}"#,
    );
    expect_response(
        &MyEnum2::Internal,
        StatusCode::INTERNAL_SERVER_ERROR,
        r#"{"error":"c"}"#,
    );
}

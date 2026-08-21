use crate::common::expect_response;
use actix_web::{http::StatusCode, ResponseError};
use std::fmt::Display;

trait MyTrait: Display {
    fn status() -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, thiserror::Error)]
#[error("my error")]
struct MyStruct;

impl MyTrait for MyStruct {}

impl ResponseError for MyStruct {}

#[derive(Debug, thiserror::Error, actix_web_error::Text)]
#[error("Error: {0}")]
#[status(transparent)]
struct MyError<T>(T);

#[derive(Debug, thiserror::Error, actix_web_error::Text)]
#[status(400)]
enum MyEnum<T: MyTrait> {
    #[error("Bad")]
    Bad,
    #[error("Delegate")]
    #[status(transparent)]
    Delegate(T),
}

#[test]
fn structs() {
    expect_response(
        &MyError(MyStruct),
        StatusCode::INTERNAL_SERVER_ERROR,
        "Error: my error",
    );
}

#[test]
fn enums() {
    expect_response(&MyEnum::<MyStruct>::Bad, StatusCode::BAD_REQUEST, "Bad");
    expect_response(
        &MyEnum::Delegate(MyStruct),
        StatusCode::INTERNAL_SERVER_ERROR,
        "Delegate",
    );
}

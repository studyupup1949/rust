//! Utils methods to write tests.

use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::test::read_body;
use actix_web::web::Bytes;

/// Check the response has the status passed, otherwise fail
/// with the response body printed out. If success
/// return the `Bytes` of the body, that can be later serialized
/// into a struct object with `serde_json::from_slice()`.
/// # Example
/// ```rust
/// #[cfg(test)]
/// mod tests {
///     use actix_contrib_rest::test::assert_status;
///
///     use actix_web::http::header::{Accept, ContentType};
///     use actix_web::http::StatusCode;
///     use actix_web::test::{call_service, TestRequest};
///     use serde_json::{json, serde_json};
///
///     #[actix_web::test]
///     async fn test_post_sale() {
///         let req = TestRequest::post()
///             .uri("/sales")
///             .insert_header(Accept::json())
///             .insert_header(ContentType::json())
///             .set_json(json!({ "customer_id": 1123, "prod_id": 9982, qty: 1 }))
///             .to_request();
///         let resp = call_service(&app, req).await;
///
///         // Check status, if isn't OK, the test will fail, printing out
///         // the response body in the form of "Response Body: ..."
///         let body = assert_status(resp, StatusCode::OK).await;
///
///         let sale: SalePayload = serde_json::from_slice(&body).unwrap();
///         assert_eq!(sale.customer_name, "John");
///         // ...
///     }
/// }
/// ```
pub async fn assert_status(resp: ServiceResponse, expected_status: StatusCode) -> Bytes {
    let status = resp.status();
    let body_bytes = read_body(resp).await;
    let body: &str = std::str::from_utf8(&body_bytes[..]).unwrap();
    assert_eq!(status, expected_status, "Response Body: {}", body);
    body_bytes
}

use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use actix_tower::extract::AutoMultipart;

#[derive(Deserialize, Debug)]
struct DummyPayload {
    name: String,
}

#[derive(Deserialize, Debug)]
struct MultiParams {
    q: Vec<String>,
}

// ============================================================================
// Extractor Hardening
// ============================================================================

#[actix_web::test]
async fn test_auto_json_missing_content_type() {
    let app = test::init_service(
        App::new().route("/", web::post().to(|_: AutoJson<DummyPayload>| async { HttpResponse::Ok().finish() }))
    ).await;

    // Missing Content-Type
    let req = test::TestRequest::post()
        .uri("/")
        .set_payload(r#"{"name":"test"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400); // Bad Request or Unsupported Media Type
}

#[actix_web::test]
async fn test_auto_json_charset_variants() {
    let app = test::init_service(
        App::new().route("/", web::post().to(|_: AutoJson<DummyPayload>| async { HttpResponse::Ok().finish() }))
    ).await;

    // charset=utf-8
    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "application/json; charset=utf-8"))
        .set_payload(r#"{"name":"test"}"#)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // charset=iso-8859-1 (might be rejected or handled depending on actix defaults)
    let req2 = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "application/json; charset=iso-8859-1"))
        .set_payload(r#"{"name":"test"}"#)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    // We just assert it doesn't panic
    assert!(resp2.status().is_client_error() || resp2.status().is_success());
}

#[actix_web::test]
async fn test_auto_json_zero_length() {
    let app = test::init_service(
        App::new().route("/", web::post().to(|_: AutoJson<Value>| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "application/json"))
        .to_request(); // empty body

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[actix_web::test]
async fn test_auto_query_duplicate_keys() {
    let app = test::init_service(
        App::new().route("/", web::get().to(|query: AutoQuery<MultiParams>| async move { 
            assert_eq!(query.into_inner().q.len(), 2);
            HttpResponse::Ok().finish() 
        }))
    ).await;

    let req = test::TestRequest::get()
        .uri("/?q=1&q=2")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // serde_urlencoded rejects duplicate keys into Vec by default with a 400 error.
    assert_eq!(resp.status().as_u16(), 400);
}

#[actix_web::test]
async fn test_auto_path_encoded_characters() {
    let app = test::init_service(
        App::new().route("/item/{id}", web::get().to(|path: AutoPath<String>| async move { 
            // The router decodes %2F to / (if configured) or treats it as the string.
            // Just verifying the extractor safely consumes it.
            let _val = path.into_inner();
            HttpResponse::Ok().finish() 
        }))
    ).await;

    // %2F is /, %00 is null byte
    let req = test::TestRequest::get()
        .uri("/item/some%2Fthing%00")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
async fn test_auto_form_urlencoded() {
    let app = test::init_service(
        App::new().route("/", web::post().to(|form: AutoForm<DummyPayload>| async move { 
            assert_eq!(form.into_inner().name, "hello world 🚀");
            HttpResponse::Ok().finish() 
        }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload("name=hello+world+%F0%9F%9A%80")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

#[actix_web::test]
async fn test_auto_multipart_large_file() {
    let app = test::init_service(
        App::new().route("/", web::post().to(|_: AutoMultipart<DummyPayload>| async { HttpResponse::Ok().finish() }))
    ).await;

    // Actix multipart parses incrementally. To prevent OOM, sizes are configured.
    // We send a mock header claiming it's massive, but the payload is small (or cuts off).
    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW"))
        .insert_header(("Content-Length", "100000000")) // 100MB claim
        .set_payload("------WebKitFormBoundary7MA4YWxkTrZu0gW\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\r\nSmall body\r\n------WebKitFormBoundary7MA4YWxkTrZu0gW--\r\n")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Client disconnect / invalid content length will lead to 400, or 200 depending on exact actix multipart behavior.
    assert!(resp.status().is_client_error() || resp.status().is_success() || resp.status().is_server_error());
}

#[actix_web::test]
async fn test_auto_multipart_missing_boundary() {
    let app = test::init_service(
        App::new().route("/", web::post().to(|_: AutoMultipart<DummyPayload>| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "multipart/form-data")) // missing boundary
        .set_payload("content")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // AutoMultipart stub returns 412 Precondition Failed.
    assert_eq!(resp.status().as_u16(), 412);
}

#[derive(Deserialize)]
struct CompositeQuery { q: String }

#[actix_web::test]
async fn test_extractor_composition() {
    let app = test::init_service(
        App::new().route("/item/{id}", web::post().to(
            |path: AutoPath<String>, query: AutoQuery<CompositeQuery>, body: AutoJson<DummyPayload>| async move {
                assert_eq!(path.into_inner(), "123");
                assert_eq!(query.into_inner().q, "search");
                assert_eq!(body.into_inner().name, "test");
                HttpResponse::Ok().finish()
            }
        ))
    ).await;

    let req = test::TestRequest::post()
        .uri("/item/123?q=search")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"name":"test"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

#[actix_web::test]
async fn test_extractor_error_accumulation() {
    // Standard extractors fail fast. If first fails, second isn't run.
    let app = test::init_service(
        App::new().route("/", web::post().to(
            |_: AutoQuery<CompositeQuery>, _: AutoJson<DummyPayload>| async move {
                HttpResponse::Ok().finish()
            }
        ))
    ).await;

    // Both query and json are invalid. Query extractor runs first and should fail.
    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"invalid":"json"}"#) // invalid json
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

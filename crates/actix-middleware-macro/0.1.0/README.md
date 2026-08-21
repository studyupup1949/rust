# actix-middleware-macro

A macro to generate actix-web middleware. Useful for the times when you can't be bothered to figure it out yourself for the 100th time.


## Usage

```rust
use actix_web::web;

    use super::*;

    create_middleware!(
        TimingCorsHeaders,
        |ctx: &MiddlewareTransform<S>, req: ServiceRequest| {
            use actix_web::http::header::{HeaderName, HeaderValue};
            use chrono::Utc;

            let start = Utc::now();

            let fut = ctx.service.call(req);
            Box::pin(async move {
                let mut res = fut.await?;
                let duration = Utc::now() - start;
                res.headers_mut().insert(
                    HeaderName::from_static("x-app-time-ms"),
                    HeaderValue::from_str(&format!("{}", duration.num_milliseconds()))?,
                );
                res.headers_mut().insert(
                    HeaderName::from_static("x-app-time-micros"),
                    HeaderValue::from_str(&format!(
                        "{}",
                        duration.num_microseconds().unwrap_or_default()
                    ))?,
                );
                // CORS header
                res.headers_mut().insert(
                    HeaderName::from_static("access-control-allow-origin"),
                    HeaderValue::from_str("*")?,
                );
                res.headers_mut().insert(
                    HeaderName::from_static("access-control-allow-methods"),
                    HeaderValue::from_str("GET, POST, OPTIONS")?,
                );
                Ok(res)
            })
        }
    );

    #[actix_web::test]
    async fn works() {
        let _server = tokio::spawn(async {
            actix_web::HttpServer::new(|| {
                actix_web::App::new()
                    .default_service(web::to(|| async { actix_web::HttpResponse::Ok() }))
                    .wrap(timing_cors_headers_middleware::Middleware)
            })
            .bind("127.1:8080")
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response = ureq::get("http://127.1:8080").call().unwrap();
        assert!(response.header("x-app-time-ms").is_some());
    }
```
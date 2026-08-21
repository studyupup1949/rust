use actix_web::{
    Error as ActixError, FromRequest, HttpRequest, dev::Payload, error::ErrorBadRequest, web::Bytes,
};
use futures_util::future::LocalBoxFuture;
use serde::de::DeserializeOwned;

/// Custom JSON extractor that uses serde_path_to_error for detailed error messages.
///
/// This is a drop-in replacement for `actix_web::web::Json<T>` that provides
/// detailed JSON path information when deserialization fails.
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: DeserializeOwned + 'static> FromRequest for Json<T> {
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let bytes_fut = Bytes::from_request(req, payload);

        Box::pin(async move {
            let bytes = bytes_fut
                .await
                .map_err(|e| ErrorBadRequest(format!("Failed to read request body: {}", e)))?;

            let jd = &mut serde_json::Deserializer::from_slice(&bytes);
            let value = serde_path_to_error::deserialize(jd).map_err(|e| {
                ErrorBadRequest(format!("Invalid JSON at {}: {}", e.path(), e.inner()))
            })?;

            Ok(Json(value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        age: u32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct NestedData {
        value: i32,
    }

    #[actix_web::test]
    async fn test_successful_deserialization() {
        async fn handler(data: Json<TestData>) -> HttpResponse {
            assert_eq!(data.name, "Test");
            assert_eq!(data.age, 20);
            HttpResponse::Ok().finish()
        }

        let app = test::init_service(App::new().route("/test", web::post().to(handler))).await;

        let payload = serde_json::json!({
            "name": "Test",
            "age": 20,
        });

        let req = test::TestRequest::post()
            .uri("/test")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_error_with_path_info() {
        async fn handler(_data: Json<TestData>) -> HttpResponse {
            HttpResponse::Ok().finish()
        }

        let app = test::init_service(App::new().route("/test", web::post().to(handler))).await;

        let payload = serde_json::json!({
            "name": "Test",
            "age": "invalid",
        });

        let req = test::TestRequest::post()
            .uri("/test")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}

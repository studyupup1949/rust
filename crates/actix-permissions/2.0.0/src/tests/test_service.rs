#[cfg(test)]
mod tests {
    use crate::builder::default_deny_handler;
    use actix_http::body::MessageBody;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::http::StatusCode;
    use actix_web::{test, web, Error, HttpRequest, HttpResponse};
    use serde::{Deserialize, Serialize};
    use std::str;

    use crate::service::PermissionService;

    async fn index() -> Result<String, Error> {
        Ok("Welcome!".to_string())
    }

    #[actix_web::test]
    async fn test_accept_all() {
        async fn accept_all(_req: HttpRequest) -> actix_web::Result<bool> {
            Ok(true)
        }
        let service_req = test::TestRequest::with_uri("/").to_srv_request();

        let service = PermissionService::new(accept_all, index, default_deny_handler);

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK)
    }

    #[actix_web::test]
    async fn test_deny_all_custom_handler() {
        fn custom_deny_handler(_req: HttpRequest) -> HttpResponse {
            HttpResponse::new(StatusCode::IM_A_TEAPOT)
        }

        async fn deny_all(_req: HttpRequest) -> actix_web::Result<bool> {
            Ok(false)
        }
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(deny_all, index, custom_deny_handler);

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::IM_A_TEAPOT)
    }

    #[actix_web::test]
    async fn test_deny_all_default_handler() {
        async fn deny_all(_req: HttpRequest) -> actix_web::Result<bool> {
            Ok(false)
        }
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(deny_all, index, default_deny_handler);

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN)
    }

    #[actix_web::test]
    async fn test_deserialization_error_happens() {
        #[derive(Clone, Debug, Deserialize)]
        struct TestStub {
            param1: bool,
        }
        async fn check_json(
            _req: HttpRequest,
            data: web::Json<TestStub>,
        ) -> actix_web::Result<bool> {
            Ok(data.param1)
        }
        let service_req = test::TestRequest::with_uri("/").to_srv_request();

        let service = PermissionService::new(check_json, index, default_deny_handler);

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST)
    }

    #[actix_web::test]
    async fn test_validates_from_payload() {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        struct TestStub {
            param1: bool,
        }
        async fn check_json(
            _req: HttpRequest,
            data: web::Json<TestStub>,
        ) -> actix_web::Result<bool> {
            Ok(data.param1)
        }
        async fn index(data: web::Json<TestStub>) -> Result<String, Error> {
            Ok(data.param1.to_string())
        }

        let service_req = test::TestRequest::post()
            .set_json(&TestStub { param1: true })
            .insert_header(("Content-type", "application/json"))
            .to_srv_request();

        let service = PermissionService::new(check_json, index, default_deny_handler);

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::OK);

        let res = response_as_string(result).await;
        assert_eq!(res, "true".to_string());
    }

    /// Test helper method for extracting response as string
    async fn response_as_string(resp: ServiceResponse) -> String {
        let body_bytes = resp.into_body().try_into_bytes().unwrap().to_vec();
        str::from_utf8(&body_bytes).unwrap().to_string()
    }
}

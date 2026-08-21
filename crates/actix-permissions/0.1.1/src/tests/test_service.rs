#[cfg(test)]
mod tests {
    use std::future::{ready, Ready};
    use std::sync::Arc;

    use actix_web::dev::{Payload, Service};
    use actix_web::{test, Error, HttpRequest, HttpResponse};

    use crate::{default_deny_handler, PermissionService, StatusCode};

    async fn index() -> Result<String, Error> {
        Ok("Welcome!".to_string())
    }

    #[actix_web::test]
    async fn test_no_permission_checks_set() {
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(Arc::new(vec![]), index, default_deny_handler);

        let result = service.call(service_req).await;

        assert!(result.is_ok())
    }

    fn deny_all(
        _req: &HttpRequest,
        _payload: &mut Payload,
    ) -> Ready<actix_web::Result<bool, actix_web::Error>> {
        ready(Ok(false))
    }

    fn custom_deny_handler(_req: &HttpRequest, _payload: &mut Payload) -> HttpResponse {
        HttpResponse::new(StatusCode::UNAUTHORIZED)
    }

    #[actix_web::test]
    async fn test_deny_all() {
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(
            Arc::new(vec![Box::new(deny_all)]),
            index,
            default_deny_handler,
        );

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::FORBIDDEN)
    }

    #[actix_web::test]
    async fn test_deny_all_custom_handler() {
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(
            Arc::new(vec![Box::new(deny_all)]),
            index,
            custom_deny_handler,
        );

        let result = service.call(service_req).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED)
    }
}

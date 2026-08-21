#[cfg(test)]
mod tests {
    use std::future::{ready, Ready};
    use std::sync::Arc;
    use actix_web::{Error, HttpRequest, test};
    use actix_web::dev::{Payload, Service};
    use crate::PermissionService;

    async fn index() -> Result<String, Error> {
        Ok("Welcome!".to_string())
    }


    #[actix_web::test]
    async fn test_no_permission_checks_set() {
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(Arc::new(vec![]), index);

        let result = service.call(service_req).await;

        assert!(result.is_ok())
    }


    fn deny_all(
        _req: &HttpRequest,
        _payload: &mut Payload,
    ) -> Ready<actix_web::Result<bool, actix_web::Error>> {
        ready(Ok(false))
    }

    #[actix_web::test]
    async fn test_deny_all() {
        let service_req = test::TestRequest::with_uri("/").to_srv_request();
        let service = PermissionService::new(Arc::new(vec![Box::new(deny_all)]), index);

        let result = service.call(service_req).await;

        assert!(result.is_ok())
    }
}

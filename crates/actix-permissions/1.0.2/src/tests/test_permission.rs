#[cfg(test)]
mod tests {
    use actix_web::web::Query;
    use actix_web::{test, FromRequest};

    use crate::permission::*;
    use crate::tests::stubs::*;

    #[actix_web::test]
    async fn test_converts_to_permission() {
        let service_req = test::TestRequest::with_uri("/?status=true").to_srv_request();
        let (req, mut payload) = service_req.into_parts();

        let result = always_true(
            req.clone(),
            Query::<MyStatus>::from_request(&req, &mut payload)
                .await
                .unwrap(),
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[actix_web::test]
    async fn test_always_true_struct() {
        let service_req = test::TestRequest::with_uri("/?status=true").to_srv_request();
        let (req, mut payload) = service_req.into_parts();

        let result = PermissionStruct {}
            .call(
                req.clone(),
                Query::<MyStatus>::from_request(&req, &mut payload)
                    .await
                    .unwrap(),
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

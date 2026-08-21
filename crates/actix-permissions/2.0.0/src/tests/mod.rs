mod test_permission;
mod test_service;
#[cfg(test)]
mod stubs {
    use crate::permission::Permission;
    use actix_web::{web, HttpRequest};
    use serde::Deserialize;
    use std::future::Future;
    use std::pin::Pin;

    #[derive(Deserialize, Debug, Clone)]
    pub struct MyStatus {
        status: bool,
    }

    pub async fn always_true(
        _req: HttpRequest,
        status: web::Query<MyStatus>,
    ) -> actix_web::Result<bool> {
        Ok(status.status)
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct PermissionStruct;
    impl Permission<web::Query<MyStatus>> for PermissionStruct {
        type Future = Pin<Box<dyn Future<Output = actix_web::Result<bool>>>>;

        fn call(&self, _req: HttpRequest, args: web::Query<MyStatus>) -> Self::Future {
            Box::pin(async move { Ok(args.status) })
        }
    }
}

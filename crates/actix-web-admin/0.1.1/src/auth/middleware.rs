use actix_session::SessionExt;
use actix_web::body::BoxBody;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Middleware that requires authentication for admin routes.
///
/// Excludes login and logout paths. Redirects unauthenticated users
/// to the login page.
pub struct RequireAuth {
    admin_prefix: String,
}

impl RequireAuth {
    pub fn new(admin_prefix: &str) -> Self {
        RequireAuth {
            admin_prefix: admin_prefix.trim_end_matches('/').to_string(),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RequireAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service: Rc::new(service),
            login_url: format!("{}/login", self.admin_prefix),
            admin_prefix: self.admin_prefix.clone(),
        }))
    }
}

pub struct RequireAuthMiddleware<S> {
    service: Rc<S>,
    login_url: String,
    admin_prefix: String,
}

impl<S> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let path = req.path().to_string();
        let login_url = self.login_url.clone();
        let admin_prefix = self.admin_prefix.clone();
        let is_public = path == format!("{admin_prefix}/login")
            || path == format!("{admin_prefix}/login/")
            || path == format!("{admin_prefix}/logout")
            || path == format!("{admin_prefix}/logout/");

        if is_public {
            return Box::pin(async move { svc.call(req).await });
        }

        let session = req.get_session();
        let is_auth = session
            .get::<String>("admin_user")
            .ok()
            .flatten()
            .is_some();

        if is_auth {
            Box::pin(async move { svc.call(req).await })
        } else {
            let response = HttpResponse::Found()
                .insert_header(("Location", login_url))
                .finish();
            Box::pin(async move { Ok(req.into_response(response)) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use actix_session::{storage::CookieSessionStore, SessionMiddleware};
    use actix_web::cookie::Key;

    async fn test_app() -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        test::init_service(
            App::new()
                .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    Key::generate(),
                ))
                .service(
                    web::scope("/admin")
                        .wrap(RequireAuth::new("/admin"))
                        .route("/login", web::get().to(|| async { HttpResponse::Ok().body("login page") }))
                        .route("/dashboard", web::get().to(|| async { HttpResponse::Ok().body("dashboard") })),
                ),
        )
        .await
    }

    #[actix_web::test]
    async fn test_public_login_page() {
        let app = test_app().await;
        let req = test::TestRequest::get().uri("/admin/login").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "login page should be public");
    }

    #[actix_web::test]
    async fn test_protected_route_redirects() {
        let app = test_app().await;
        let req = test::TestRequest::get()
            .uri("/admin/dashboard")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            302,
            "unauthenticated requests should redirect"
        );
        let location = resp.response().headers().get("Location");
        assert!(location.is_some(), "should have Location header");
        assert_eq!(
            location.unwrap().to_str().unwrap(),
            "/admin/login",
            "should redirect to login"
        );
    }

}

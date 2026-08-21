use crate::Permission;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::{dev, FromRequest, Handler, HttpResponse, Responder};
use futures_core::future::LocalBoxFuture;
use std::convert::Infallible;
use std::marker::PhantomData;
use std::sync::Arc;

///
/// Service that intercepts request, validates it with a list of permissions.
/// If any of the permissions fail, 403 forbidden is returned.
/// If permissions succeed, request is proxied to handler
///
/// # Properties
/// * `perms` - list of permissions
/// * `handler` - handler, a function that returns http (serializable) response
/// * `phantom_data` - phantom data, needed to avoid warnings of unused `Args`
pub struct PermissionService<F, Args>
where
    F: Handler<Args>,
    Args: FromRequest,
    F::Output: Responder,
{
    perms: Arc<Vec<Box<dyn Permission>>>,
    handler: F,
    phantom_data: PhantomData<Args>,
}

impl<F, Args> PermissionService<F, Args>
where
    F: Handler<Args>,
    Args: FromRequest,
    F::Output: Responder,
{
    pub fn new(perms: Arc<Vec<Box<dyn Permission>>>, handler: F) -> Self {
        Self {
            perms,
            handler,
            phantom_data: PhantomData::default(),
        }
    }
}

impl<F, Args> Service<ServiceRequest> for PermissionService<F, Args>
where
    F: Handler<Args>,
    Args: FromRequest,
    F::Output: Responder,
{
    type Response = ServiceResponse;
    type Error = Infallible;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::always_ready!();

    fn call(&self, args: ServiceRequest) -> Self::Future {
        let (req, mut payload) = args.into_parts();
        let perms = Arc::clone(&self.perms);
        let handler = self.handler.clone();

        Box::pin(async move {
            for permission in perms.iter() {
                let result = permission.call(&req, &mut payload).await;
                match result {
                    Ok(false) => {
                        let response = HttpResponse::new(StatusCode::FORBIDDEN); // TODO: Forbidden or Unauthorized?
                        return Ok(ServiceResponse::new(req, response));
                    }
                    Err(err) => {
                        return Ok(ServiceResponse::from_err(err, req));
                    }
                    Ok(_) => {
                        // Do nothing
                    }
                }
            }

            let res = match Args::from_request(&req, &mut payload).await {
                Err(err) => HttpResponse::from_error(err),

                Ok(data) => handler
                    .call(data)
                    .await
                    .respond_to(&req)
                    .map_into_boxed_body(),
            };

            Ok(ServiceResponse::new(req, res))
        })
    }
}

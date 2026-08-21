//! [`PermissionService`] struct and `Service<ServiceRequest>` implementation.
use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::permission::Permission;
use actix_web::dev::*;
use actix_web::*;

/// Service that intercepts a request and validates it with a permission.
/// If permission fails, `deny_handler` is called.
/// If permission succeeds, request is proxied to `handler`.
///
/// # Properties
/// * `handler` - handler, a function that returns http (serializable) response.
/// * `pd_handler` - marker for handler type args, needed to avoid warnings of unused `Args`.
/// * `deny_handler` - function argument, called when permission check is false.
/// * `permission` - permission
/// * `pd_permission` - marker for permission type args, needed to avoid warnings of unused `P1Args`.
/// * `ready` - flag that tells if future in `call` is completed or pending.
pub struct PermissionService<F, Args, P1, P1Args>
where
    F: Handler<Args>,
    Args: FromRequest,
    F::Output: Responder,
    P1: Permission<P1Args>,
    P1Args: FromRequest,
{
    handler: F,
    pd_handler: PhantomData<Args>,
    deny_handler: fn(HttpRequest) -> HttpResponse,
    permission: P1,
    pd_permission: PhantomData<P1Args>,
    ready: Arc<AtomicBool>,
}

impl<F, Args, P1, P1Args> PermissionService<F, Args, P1, P1Args>
where
    F: Handler<Args>,
    Args: FromRequest,
    P1: Permission<P1Args>,
    P1Args: FromRequest,
    F::Output: Responder,
{
    /// Creates new `PermissionService`.
    pub fn new(permission: P1, handler: F, deny_handler: fn(HttpRequest) -> HttpResponse) -> Self {
        Self {
            handler,
            pd_handler: Default::default(),
            deny_handler,
            permission,
            pd_permission: Default::default(),
            ready: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<F, Args, P1, P1Args> Service<ServiceRequest> for PermissionService<F, Args, P1, P1Args>
where
    F: Handler<Args>,
    Args: FromRequest,
    P1: Permission<P1Args>,
    P1Args: FromRequest,
    F::Output: Responder,
{
    type Response = ServiceResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, _: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        if self.ready.load(Ordering::Relaxed) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn call(&self, args: ServiceRequest) -> Self::Future {
        let handler = self.handler.clone();
        let deny_handler = self.deny_handler;
        let permission = self.permission.clone();
        let ready = self.ready.clone();
        Box::pin(async move {
            let (req, mut payload) = args.into_parts();

            let service_response = match P1Args::from_request(&req, &mut payload).await {
                Err(err) => ServiceResponse::new(req, HttpResponse::from_error(err)),
                Ok(data) => {
                    let permission_check_result = permission.call(req.clone(), data).await;
                    match permission_check_result {
                        Ok(true) => match Args::from_request(&req, &mut payload).await {
                            Err(err) => ServiceResponse::new(req, HttpResponse::from_error(err)),
                            Ok(data) => {
                                let handler_response = handler
                                    .call(data)
                                    .await
                                    .respond_to(&req)
                                    .map_into_boxed_body();
                                ServiceResponse::new(req, handler_response)
                            }
                        },
                        Ok(false) => {
                            let response = deny_handler(req.clone());
                            ServiceResponse::new(req, response)
                        }
                        Err(err) => ServiceResponse::from_err(err, req),
                    }
                }
            };

            ready.store(true, Ordering::Relaxed);
            Ok(service_response)
        })
    }
}

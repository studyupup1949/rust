//! Future types for the Tower bridge.

use std::{future::Future, pin::Pin};

use actix_web::{body::BoxBody, dev::ServiceResponse, Error};

use crate::compat::tower::body::ActixResponseBody;

/// Future for the [`TowerMiddlewareService`](super::service::TowerMiddlewareService).
pub type TowerMiddlewareFuture = Pin<Box<dyn Future<Output = Result<ServiceResponse, Error>>>>;

/// Future for the [`ActixServiceWrapper`](super::service::ActixServiceWrapper).
pub type ActixServiceWrapperFuture = Pin<
    Box<
        dyn Future<
            Output = Result<
                http::Response<ActixResponseBody<BoxBody>>,
                crate::internal::common::BoxError,
            >,
        >,
    >,
>;

/*
 * This file is part of Actix Form Data.
 *
 * Copyright © 2020 Riley Trautman
 *
 * Actix Form Data is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Actix Form Data is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Actix Form Data.  If not, see <http://www.gnu.org/licenses/>.
 */

use crate::{
    types::{Form, Value},
    upload::handle_multipart,
};
use actix_web::{
    dev::{Payload, Service, ServiceRequest, Transform},
    http::StatusCode,
    FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    task::{Context, Poll},
};
use tokio::sync::oneshot::{channel, Receiver};

#[derive(Debug, thiserror::Error)]
pub enum FromRequestError {
    #[error("Uploaded guard used without Multipart middleware")]
    MissingMiddleware,
    #[error("Impossible Error! Middleware exists, didn't fail, and didn't send value")]
    TxDropped,
}

impl ResponseError for FromRequestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingMiddleware | Self::TxDropped => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::MissingMiddleware | Self::TxDropped => {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

struct Uploaded<T> {
    rx: Receiver<Value<T>>,
}

pub struct MultipartMiddleware<S, T, E> {
    form: Form<T, E>,
    service: S,
}

impl<T> FromRequest for Value<T>
where
    T: 'static,
{
    type Error = FromRequestError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let opt = req.extensions_mut().remove::<Uploaded<T>>();
        Box::pin(async move {
            let fut = opt.ok_or(FromRequestError::MissingMiddleware)?;

            fut.rx.await.map_err(|_| FromRequestError::TxDropped)
        })
    }
}

impl<S, T, E> Transform<S, ServiceRequest> for Form<T, E>
where
    S: Service<ServiceRequest, Error = actix_web::Error>,
    S::Future: 'static,
    T: 'static,
    E: ResponseError + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = MultipartMiddleware<S, T, E>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MultipartMiddleware {
            form: self.clone(),
            service,
        }))
    }
}

impl<S, T, E> Service<ServiceRequest> for MultipartMiddleware<S, T, E>
where
    S: Service<ServiceRequest, Error = actix_web::Error>,
    S::Future: 'static,
    T: 'static,
    E: ResponseError + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<S::Response, S::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let (tx, rx) = channel();
        req.extensions_mut().insert(Uploaded { rx });
        let payload = req.take_payload();
        let multipart = actix_multipart::Multipart::new(req.headers(), payload);
        let form = self.form.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let uploaded = match handle_multipart(multipart, form.clone()).await {
                Ok(Ok(uploaded)) => uploaded,
                Ok(Err(e)) => return Err(e.into()),
                Err(e) => {
                    if let Some(f) = form.transform_error.clone() {
                        return Err((f)(e));
                    } else {
                        return Err(e.into());
                    }
                }
            };
            let _ = tx.send(uploaded);
            fut.await
        })
    }
}

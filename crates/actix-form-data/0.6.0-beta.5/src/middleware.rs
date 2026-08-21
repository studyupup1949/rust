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
    error::Error,
    types::{Form, Value},
    upload::handle_multipart,
};
use actix_web::{
    dev::{Payload, Service, ServiceRequest, Transform},
    FromRequest, HttpMessage, HttpRequest,
};
use std::{
    future::{Ready, ready, Future},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::oneshot::{channel, Receiver};

struct Uploaded {
    rx: Receiver<Value>,
}

pub struct MultipartMiddleware<S> {
    form: Form,
    service: S,
}

impl FromRequest for Value {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let opt = req.extensions_mut().remove::<Uploaded>();
        Box::pin(async move {
            let fut = opt.ok_or(Error::MissingMiddleware)?;

            fut.rx.await.map_err(|_| Error::TxDropped)
        })
    }
}

impl<S> Transform<S, ServiceRequest> for Form
where
    S: Service<ServiceRequest, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = MultipartMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MultipartMiddleware {
            form: self.clone(),
            service,
        }))
    }
}

impl<S> Service<ServiceRequest> for MultipartMiddleware<S>
where
    S: Service<ServiceRequest, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>>>>;

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
                Ok(uploaded) => uploaded,
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

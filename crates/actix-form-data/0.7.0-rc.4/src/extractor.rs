/*
 * This file is part of Actix Form Data.
 *
 * Copyright © 2026 asonix
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
use actix_web::{dev::Payload, FromRequest, HttpRequest, ResponseError};
use std::{future::Future, pin::Pin, rc::Rc};

pub trait FormData {
    type Item: 'static;
    type Error: ResponseError + 'static;

    fn form(req: &HttpRequest) -> Result<Form<Self::Item, Self::Error>, Self::Error>;

    fn extract(value: Value<Self::Item>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub struct Multipart<T>(pub T);

impl<T> FromRequest for Multipart<T>
where
    T: FormData,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + 'static>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let multipart = actix_multipart::Multipart::new(req.headers(), payload.take());
        let res = T::form(req);

        Box::pin(async move {
            let form = Rc::new(res?);

            let uploaded = match handle_multipart(multipart, Rc::clone(&form)).await {
                Ok(Ok(uploaded)) => uploaded,
                Ok(Err(e)) => return Err(e.into()),
                Err(e) => {
                    if let Some(f) = &form.transform_error {
                        return Err((f)(e));
                    } else {
                        return Err(e.into());
                    }
                }
            };

            Ok(Multipart(T::extract(uploaded)?))
        })
    }
}

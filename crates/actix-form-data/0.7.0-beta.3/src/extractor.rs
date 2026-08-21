use crate::{
    types::{Form, Value},
    upload::handle_multipart,
};
use actix_web::{dev::Payload, FromRequest, HttpRequest, ResponseError};
use std::{future::Future, pin::Pin, rc::Rc};

pub trait FormData {
    type Item: 'static;
    type Error: ResponseError + 'static;

    fn form(req: &HttpRequest) -> Form<Self::Item, Self::Error>;

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
        let form = Rc::new(T::form(req));

        Box::pin(async move {
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

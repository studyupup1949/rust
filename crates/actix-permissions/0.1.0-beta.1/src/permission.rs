use actix_web::dev::*;
use actix_web::*;
use std::future::Ready;

pub trait Permission: CloneablePermission {
    fn call(&self, req: &HttpRequest, payload: &mut Payload) -> Ready<actix_web::Result<bool>>;
}

/// CloneablePermission trait is needed for cloning a boxed trait object
pub trait CloneablePermission {
    fn box_clone(&self) -> Box<dyn Permission>;
}

impl<T> CloneablePermission for T
where
    T: Permission + Clone + 'static,
{
    fn box_clone(&self) -> Box<dyn Permission> {
        Box::new(self.clone())
    }
}

/// Magic that allows function as argument, instead of a Permission trait
impl<Func> Permission for Func
where
    Func: Fn(&HttpRequest, &mut Payload) -> Ready<actix_web::Result<bool>>,
    Func: Clone,
    Func: 'static,
{
    #[inline]
    #[allow(non_snake_case)]
    fn call(&self, req: &HttpRequest, p: &mut Payload) -> Ready<actix_web::Result<bool>> {
        (self)(req, p)
    }
}

impl Clone for Box<dyn Permission> {
    fn clone(&self) -> Box<dyn Permission> {
        self.box_clone()
    }
}

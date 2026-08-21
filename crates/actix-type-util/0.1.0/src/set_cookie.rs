use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::{HttpRequest, HttpResponse, Responder};
use std::borrow::Cow;

pub struct SetCookie<'c, T: Responder> {
    inner: T,
    cookie: Cookie<'c>,
}

impl<'c, T: Responder> SetCookie<'c, T> {
    pub fn new<N, V>(
        inner: T,
        cookie_name: N,
        cookie_value: V
    ) -> Self
    where
        N: Into<Cow<'c, str>>,
        V: Into<Cow<'c, str>>,
    {
        Self {
            inner,
            cookie: Self::create_cookie(
                cookie_name,
                cookie_value,
                "/",
                true,
                Expiration::Session,
                Some(SameSite::None),
                true,
            )
        }
    }

    pub fn new_with_opts<N, V, P>(
        inner: T,
        cookie_name: N,
        cookie_value: V,
        path: P,
        http_only: bool,
        expires: Expiration,
        same_site: Option<SameSite>,
        secure: bool,
    ) -> Self
    where
        N: Into<Cow<'c, str>>,
        V: Into<Cow<'c, str>>,
        P: Into<Cow<'c, str>>
    {
        Self {
            inner,
            cookie: Self::create_cookie(
                cookie_name,
                cookie_value,
                path,
                http_only,
                expires,
                same_site,
                secure,
            )
        }
    }

    fn create_cookie<'c1, N, V, P>(
        cookie_name: N,
        cookie_value: V,
        path: P,
        http_only: bool,
        expires: Expiration,
        same_site: Option<SameSite>,
        secure: bool,
    ) -> Cookie<'c1>
    where
        N: Into<Cow<'c1, str>>,
        V: Into<Cow<'c1, str>>,
        P: Into<Cow<'c1, str>>
    {
        let mut cookie = Cookie::new(cookie_name, cookie_value);
        cookie.set_path(path);
        cookie.set_http_only(http_only);
        cookie.set_expires(expires);
        cookie.set_same_site(same_site);
        cookie.set_secure(secure);

        cookie
    }
}

impl<'c, T: Responder> Responder for SetCookie<'c, T> {
    type Body = <T as Responder>::Body;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut response = self.inner.respond_to(req);
        response.add_cookie(&self.cookie).unwrap();
        response
    }
}

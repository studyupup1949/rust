use reqwest::{
    Method, Url,
    blocking::{Client, RequestBuilder, Response},
    header::HeaderMap,
};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

use super::super::shared::CustomizedRequest;

/// `Accepts<T>` implementation that posts serialized values via HTTP.
#[must_use = "HttpClientAcceptor must be used to dispatch HTTP requests"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct HttpClientAcceptor<NextAccepts>
where
    NextAccepts: Accepts<reqwest::Result<Response>>,
{
    client: Client,
    url: Url,
    headers: HeaderMap,
    method: Method,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<NextAccepts> HttpClientAcceptor<NextAccepts>
where
    NextAccepts: Accepts<reqwest::Result<Response>>,
{
    /// Creates a `HttpClientAcceptor` that uses `POST` method by default.
    pub fn new(client: Client, url: Url, headers: HeaderMap, next_acceptor: NextAccepts) -> Self {
        Self {
            client,
            url,
            headers,
            method: Method::POST,
            next_acceptor,
        }
    }

    /// Creates a `HttpClientAcceptor` that uses a specified HTTP method.
    pub fn with_method(
        client: Client,
        url: Url,
        headers: HeaderMap,
        method: Method,
        next_acceptor: NextAccepts,
    ) -> Self {
        Self {
            client,
            url,
            headers,
            method,
            next_acceptor,
        }
    }

    fn send_request<RequestBody, F>(&self, body: RequestBody, customize: F)
    where
        RequestBody: Into<reqwest::blocking::Body>,
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        let mut request = self.client.request(self.method.clone(), self.url.clone());
        if !self.headers.is_empty() {
            request = request.headers(self.headers.clone());
        }

        let request = customize(request).body(body);
        let result = request.send();

        self.next_acceptor.accept(result);
    }
}

impl<NextAccepts, RequestBody> Accepts<RequestBody> for HttpClientAcceptor<NextAccepts>
where
    NextAccepts: Accepts<reqwest::Result<Response>>,
    RequestBody: Into<reqwest::blocking::Body>,
{
    fn accept(&self, body: RequestBody) {
        self.send_request(body, |r| r);
    }
}

impl<NextAccepts, RequestBody, F> Accepts<CustomizedRequest<RequestBody, F>>
    for HttpClientAcceptor<NextAccepts>
where
    NextAccepts: Accepts<reqwest::Result<Response>>,
    RequestBody: Into<reqwest::blocking::Body>,
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    fn accept(&self, req: CustomizedRequest<RequestBody, F>) {
        let (body, customize) = req.into_parts();
        self.send_request(body, customize);
    }
}

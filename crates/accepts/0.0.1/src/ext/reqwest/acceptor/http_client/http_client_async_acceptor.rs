use core::future::Future;

use reqwest::{Body, Client, Method, RequestBuilder, Response, Url, header::HeaderMap};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

use super::super::shared::CustomizedRequest;

/// `Accepts<T>` implementation that posts serialized values via HTTP.
#[must_use = "HttpClientAsyncAcceptor must be used to dispatch async HTTP requests"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct HttpClientAsyncAcceptor<NextAccepts>
where
    NextAccepts: AsyncAccepts<reqwest::Result<Response>>,
{
    client: Client,
    url: Url,
    headers: HeaderMap,
    method: Method,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<NextAccepts> HttpClientAsyncAcceptor<NextAccepts>
where
    NextAccepts: AsyncAccepts<reqwest::Result<Response>>,
{
    /// Creates a `HttpClientAsyncAcceptor` that uses `POST` method by default.
    pub fn new(client: Client, url: Url, headers: HeaderMap, next_acceptor: NextAccepts) -> Self {
        Self {
            client,
            url,
            headers,
            method: Method::POST,
            next_acceptor,
        }
    }

    /// Creates a `HttpClientAsyncAcceptor` that uses a specified HTTP method.
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

    fn send_request<'a, RequestBody, F>(
        &'a self,
        body: RequestBody,
        customize: F,
    ) -> impl Future<Output = ()> + 'a
    where
        RequestBody: Into<Body> + 'a,
        F: FnOnce(RequestBuilder) -> RequestBuilder + 'a,
    {
        async {
            let mut request = self.client.request(self.method.clone(), self.url.clone());
            if !self.headers.is_empty() {
                request = request.headers(self.headers.clone());
            }

            let request = customize(request).body(body);
            let result = request.send().await;

            self.next_acceptor.accept_async(result).await;
        }
    }
}

impl<NextAccepts, RequestBody> AsyncAccepts<RequestBody> for HttpClientAsyncAcceptor<NextAccepts>
where
    NextAccepts: AsyncAccepts<reqwest::Result<Response>>,
    RequestBody: Into<Body>,
{
    fn accept_async<'a>(&'a self, body: RequestBody) -> impl Future<Output = ()> + 'a
    where
        RequestBody: 'a,
    {
        self.send_request(body, |r| r)
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<NextAccepts, RequestBody, F> AsyncAccepts<CustomizedRequest<RequestBody, F>>
    for HttpClientAsyncAcceptor<NextAccepts>
where
    NextAccepts: AsyncAccepts<reqwest::Result<Response>>,
    RequestBody: Into<Body>,
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    fn accept_async<'a>(
        &'a self,
        req: CustomizedRequest<RequestBody, F>,
    ) -> impl Future<Output = ()> + 'a
    where
        CustomizedRequest<RequestBody, F>: 'a,
    {
        let (body, customize) = req.into_parts();
        self.send_request(body, customize)
    }
}

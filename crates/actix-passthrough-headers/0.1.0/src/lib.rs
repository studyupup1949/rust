use actix_web::{
	Error,
	dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
	http::header::{HeaderName, HeaderValue},
};
use std::pin::Pin;
use std::{
	future::{Ready, ready},
	rc::Rc,
};

type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// # Actix-passthrough-headers
///
/// Middleware to passthrough headers. It will be useful when you want to pass header from request to
/// response without changes.
///
/// ## Installation
///
/// ```bash
/// cargo add actix-passthrough-headers
/// ```
///
/// ## Usage
///
/// ```rs
/// App::new().wrap(PassthroughHeaders::new(vec!["X-Request-Id"]))
/// ```
///
/// ## Options
///
/// By default when route returns some header:
///
/// ```rs
/// async fn route() -> HttpResponse {
///     HttpResponse::Ok()
///         .insert_header(("X-Request-Id", "2"))
///         .finish()
/// }
/// ```
///
/// And the same header was passed to middleware:
///
/// ```rs
/// App::new().wrap(PassthroughHeaders::new(["X-Request-Id"]))
/// ```
///
/// The middleware ignores route's header and returns initial header value. It means when http request
/// has header `X-Request-Id: 1` and route returns `X-Request-Id: 2`, response header will be
/// `X-Request-Id: 1`.
///
/// ### Method preserve_response_headers()
///
/// Method `preserve_response_headers` disallow to rewrite route's headers. It means when http request
/// has header `X-Request-Id: 1` and route returns `X-Request-Id: 2`, response header will be
/// `X-Request-Id: 2`.
///
/// ```rs
/// App::new().wrap(PassthroughHeaders::new(vec!["X-Request-Id"]).preserve_response_headers())
/// ```
///
/// ## Examples
///
/// ### Simple Example
///
/// ```rs
/// use actix_web::{App, HttpServer};
/// use actix_passthrough_headers::PassthroughHeaders;
///
/// #[actix_web::main]
/// async fn main() {
///     HttpServer::new(|| {
///         App::new().wrap(PassthroughHeaders::new(vec!["X-Request-Id"]))
///     })
///         .bind(("127.0.0.1", 8080)).unwrap()
///         .run()
///         .await;
/// }
/// ```
///
/// ### Preserve response headers example
///
/// ```rs
/// use actix_web::{App, HttpServer};
/// use actix_passthrough_headers::PassthroughHeaders;
///
/// async fn route() -> HttpResponse {
///     HttpResponse::Ok().insert_header(("X-Request-Id", "1")).finish()
/// }
///
/// #[actix_web::main]
/// async fn main() {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(PassthroughHeaders::new(vec!["X-Request-Id"]).preserve_response_headers())
///             .route("/", web::get().to(route))
///     })
///         .bind(("127.0.0.1", 8080)).unwrap()
///         .run()
///         .await;
/// }
/// ```
pub struct PassthroughHeaders {
	headers: Rc<Vec<HeaderName>>,
	rewrite_headers: Rc<bool>,
}

impl PassthroughHeaders {
	pub fn new<S: AsRef<str>, H: IntoIterator<Item = S>>(headers: H) -> Self {
		Self {
			headers: Rc::new(
				// Actix web works using lower case headers. We allow to pass to constructor any
				// case of header. So code below transforms headers to lower case.
				headers.into_iter().filter_map(|s| HeaderName::try_from(s.as_ref()).ok()).collect(),
			),
			rewrite_headers: Rc::new(true),
		}
	}

	/// Disallow to rewrite existed headers.
	///
	/// ### Default behavior
	///
	/// By default when route returns some header:
	/// ```rs
	/// async fn route() -> HttpResponse {
	///     HttpResponse::Ok()
	///         .insert_header(("X-Request-Id", "2"))
	///         .finish()
	/// }
	/// ```
	/// And the same header was passed to middleware:
	/// ```rs
	/// App::new().wrap(PassthroughHeaders::new(["X-Request-Id"]))
	/// ```
	/// The middleware ignores route's header and returns initial header value. It means when http
	/// request has header `X-Request-Id: 1` and route returns `X-Request-Id: 2`, response header
	/// will be `X-Request-Id: 1`.
	///
	/// ### Preserved behavior
	///
	/// The method `preserve_response_headers` disallow to rewrite route's headers. It means when
	/// http request has header `X-Request-Id: 1` and route returns `X-Request-Id: 2`, response
	/// header will be `X-Request-Id: 2`.
	pub fn preserve_response_headers(mut self) -> Self {
		self.rewrite_headers = Rc::new(false);
		self
	}
}

impl<S, B> Transform<S, ServiceRequest> for PassthroughHeaders
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
	S::Future: 'static,
	B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = Error;
	type InitError = ();
	type Transform = PassthroughHeadersMiddleware<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(PassthroughHeadersMiddleware {
			service,
			headers: Rc::clone(&self.headers),
			rewrite_headers: Rc::clone(&self.rewrite_headers),
		}))
	}
}

pub struct PassthroughHeadersMiddleware<S> {
	service: S,
	headers: Rc<Vec<HeaderName>>,
	rewrite_headers: Rc<bool>,
}

impl<S, B> Service<ServiceRequest> for PassthroughHeadersMiddleware<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
	S::Future: 'static,
	B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = Error;
	type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	forward_ready!(service);

	fn call(&self, req: ServiceRequest) -> Self::Future {
		let request_headers = req.headers();

		let headers: Vec<(HeaderName, Vec<HeaderValue>)> = self
			.headers
			.iter()
			.filter_map(|name| {
				let values: Vec<HeaderValue> = request_headers.get_all(name).cloned().collect();
				if values.is_empty() { None } else { Some((name.clone(), values)) }
			})
			.collect();
		let rewrite_headers = Rc::clone(&self.rewrite_headers);

		let fut = self.service.call(req);

		Box::pin(async move {
			let mut res = fut.await?;
			let muted_headers = res.headers_mut();

			for (key, values) in headers {
				// If disallows to rewrite existed headers, do nothing
				if !*rewrite_headers && muted_headers.contains_key(&key) {
					continue;
				}

				let mut iter = values.into_iter();

				if let Some(first) = iter.next() {
					// insert deletes existed values
					muted_headers.insert(key.clone(), first);

					for value in iter {
						muted_headers.append(key.clone(), value);
					}
				}
			}

			Ok(res)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::PassthroughHeaders;
	use actix_web::{
		App, HttpResponse,
		test::{TestRequest, call_service, init_service},
		web,
	};

	async fn route() -> HttpResponse {
		HttpResponse::Ok().finish()
	}

	#[actix_web::test]
	async fn when_headers_do_not_passed() {
		let v: Vec<&str> = Vec::new();
		let app = init_service(
			App::new().wrap(PassthroughHeaders::new(v)).route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get().uri("/");
		let res = call_service(&app, req.to_request()).await;

		assert!(res.headers().is_empty());
	}

	#[actix_web::test]
	async fn when_headers_passed() {
		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(["X-Request-Id", "X-Abc"]))
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get()
			.uri("/")
			.insert_header(("X-Request-Id", ""))
			.insert_header(("X-Abc", "hello"));

		let res = call_service(&app, req.to_request()).await;

		assert_eq!(res.headers().get("X-Request-Id").unwrap(), "");
		assert_eq!(res.headers().get("X-Abc").unwrap(), "hello");
	}

	#[actix_web::test]
	async fn when_headers_passed_to_config_and_do_not_passed_to_headers() {
		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(["X-Request-Id"]))
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get().uri("/").insert_header(("Another-Header", "123"));
		let res = call_service(&app, req.to_request()).await;

		assert!(res.headers().get("X-Request-Id").is_none());
		assert!(res.headers().get("Another-Header").is_none())
	}

	#[actix_web::test]
	async fn when_headers_passed_and_header_was_changed_within_controller() {
		async fn route() -> HttpResponse {
			HttpResponse::Ok().insert_header(("X-Request-Id", "2")).finish()
		}

		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(["X-Request-Id"]))
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get().uri("/").insert_header(("X-Request-Id", "1"));
		let res = call_service(&app, req.to_request()).await;

		assert_eq!(res.headers().get("X-Request-Id").unwrap(), "1");
	}

	#[actix_web::test]
	async fn when_headers_passed_and_preserved_response_headers_and_header_was_changed_within_controller()
	 {
		async fn route() -> HttpResponse {
			HttpResponse::Ok().insert_header(("X-Request-Id", "2")).finish()
		}

		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(["X-Request-Id"]).preserve_response_headers())
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get().uri("/").insert_header(("X-Request-Id", "1"));
		let res = call_service(&app, req.to_request()).await;

		assert_eq!(res.headers().get("X-Request-Id").unwrap(), "2");
	}

	#[actix_web::test]
	async fn when_headers_do_not_passed_and_headers_returned_from_controller() {
		async fn route() -> HttpResponse {
			HttpResponse::Ok()
				.insert_header(("X-Request-Id", "1"))
				.insert_header(("X-Second", "2"))
				.finish()
		}

		let v: Vec<&str> = Vec::new();
		let app = init_service(
			App::new().wrap(PassthroughHeaders::new(v)).route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get().uri("/");
		let res = call_service(&app, req.to_request()).await;

		assert_eq!(res.headers().get("X-Request-Id").unwrap(), "1");
		assert_eq!(res.headers().get("X-Second").unwrap(), "2");
	}

	#[actix_web::test]
	async fn when_headers_do_not_passed_and_preserved_response_headers() {
		async fn route() -> HttpResponse {
			HttpResponse::Ok()
				.insert_header(("X-Request-Id", "1"))
				.insert_header(("X-Second", "2"))
				.finish()
		}

		let v: Vec<&str> = Vec::new();
		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(v).preserve_response_headers())
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get().uri("/");
		let res = call_service(&app, req.to_request()).await;

		assert_eq!(res.headers().get("X-Request-Id").unwrap(), "1");
		assert_eq!(res.headers().get("X-Second").unwrap(), "2");
	}

	#[actix_web::test]
	async fn when_headers_passed_and_header_multi_value() {
		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(["X-Request-Id"]).preserve_response_headers())
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get()
			.uri("/")
			.insert_header(("X-Request-Id", "1"))
			.append_header(("X-Request-Id", "2"));
		let res = call_service(&app, req.to_request()).await;
		let mut headers = res.headers().get_all("X-Request-Id");

		assert_eq!(headers.next().unwrap(), "1");
		assert_eq!(headers.next().unwrap(), "2");
	}

	#[actix_web::test]
	async fn when_headers_passed_and_preserved_response_headers_header_multi_value() {
		async fn route() -> HttpResponse {
			HttpResponse::Ok()
				.insert_header(("X-One", "one"))
				.insert_header(("X-Request-Id", "3"))
				.append_header(("X-Request-Id", "4"))
				.append_header(("X-Request-Id", "5"))
				.finish()
		}

		let app = init_service(
			App::new()
				.wrap(PassthroughHeaders::new(["X-Request-Id"]).preserve_response_headers())
				.route("/", web::get().to(route)),
		)
		.await;

		let req = TestRequest::get()
			.uri("/")
			.insert_header(("X-Request-Id", "1"))
			.append_header(("X-Request-Id", "2"));
		let res = call_service(&app, req.to_request()).await;
		let mut headers = res.headers().get_all("X-Request-Id");

		assert_eq!(headers.next().unwrap(), "3");
		assert_eq!(headers.next().unwrap(), "4");
		assert_eq!(headers.next().unwrap(), "5");
		assert_eq!(res.headers().get("X-One").unwrap(), "one");
	}
}

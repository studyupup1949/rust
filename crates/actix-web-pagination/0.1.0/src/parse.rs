use super::pagination::Pagination;
use super::pagination_config::PaginationConfig;

use actix_web::dev;
use actix_web::web;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use futures_util::future;

impl FromRequest for Pagination {
    type Error = actix_web::Error;

    type Future = future::Ready<actix_web::Result<Self>>;

    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        future::ready(Pagination::parse(req))
    }
}

impl Pagination {
    pub(crate) fn parse(req: &HttpRequest) -> actix_web::Result<Pagination> {
        let config = req
            .app_data_resolved()
            .unwrap_or(&PaginationConfig::DEFAULT);

        let mut it = config.pagination();
        form_urlencoded::parse(req.query_string().as_bytes()).for_each(|(k, v)| match k {
            _ if k == config.page_name => it.page = config.parse_page(&v) as _,
            _ if k == config.per_page_name => it.per_page = config.parse_per_page(&v) as _,
            _ => {}
        });

        Ok(it)
    }
}

trait HttpRequestExt {
    fn app_data_resolved<T: 'static>(&self) -> Option<&T>;
}

impl HttpRequestExt for HttpRequest {
    fn app_data_resolved<T: 'static>(&self) -> Option<&T> {
        self.app_data()
            .or_else(|| self.app_data::<web::Data<T>>().map(|it| it.as_ref()))
    }
}

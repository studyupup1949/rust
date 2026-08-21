use actix_web::{get, web, HttpRequest, HttpResponse};

use super::utils;
use super::SrvResult;

#[get("/translate/{app}/")]
pub async fn translate(app: web::Path<String>, req: HttpRequest) -> SrvResult {
    Ok(HttpResponse::Ok().body(format!("App: {}, Lang: {}", app, utils::get_lang(&req))))
}

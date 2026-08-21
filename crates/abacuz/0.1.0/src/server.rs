use actix_files::NamedFile;
use actix_web::error::Error;
use actix_web::{get, HttpRequest, HttpResponse};

use super::precept;
use super::SrvData;
use super::SrvResult;

#[get("/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().body("Abacuz pong.")
}

#[get("/favicon.ico")]
pub async fn favicon() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("./favicon.ico")?)
}

#[get("/version")]
async fn version() -> HttpResponse {
    HttpResponse::Ok().body(format!(
        "{}{}",
        "Abacuz version: ",
        clap::crate_version!()
    ))
}

#[get("/stop")]
pub async fn stop(req: HttpRequest, srv_data: SrvData) -> SrvResult {
    precept::stop(&req, &srv_data)
}

#[get("/shut")]
pub async fn shut(req: HttpRequest, srv_data: SrvData) -> SrvResult {
    precept::shut(&req, &srv_data)
}

use actix_files::NamedFile;
use actix_web::error::{Error, ErrorForbidden};
use actix_web::{post, web::Json, HttpRequest};

use std::path::Path;

use super::data::OnePath;
use super::data::PathData;
use super::data::TwoPath;
use super::dirs;
use super::files;
use super::guard;
use super::utils;
use super::SrvData;
use super::SrvResult;

#[post("/dir/list")]
pub async fn dir_list(one: Json<OnePath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &one.path);
    guard::check_dir_access(&path, None, "/dir/list", &user)?;
    dirs::list(Path::new(&path).to_owned())
}

#[post("/dir/new")]
pub async fn dir_new(one: Json<OnePath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &one.path);
    guard::check_dir_access(&path, None, "/dir/new", &user)?;
    dirs::new(Path::new(&path).to_owned())
}

#[post("/dir/copy")]
pub async fn dir_copy(two: Json<TwoPath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let origin = utils::get_absolute(&user, &two.origin);
    let destiny = utils::get_absolute(&user, &two.destiny);
    guard::check_dir_access(&origin, Some(&destiny), "/dir/copy", &user)?;
    dirs::copy(
        Path::new(&origin).to_owned(),
        Path::new(&destiny).to_owned(),
    )
}

#[post("/dir/move")]
pub async fn dir_move(two: Json<TwoPath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let origin = utils::get_absolute(&user, &two.origin);
    let destiny = utils::get_absolute(&user, &two.destiny);
    guard::check_dir_access(&origin, Some(&destiny), "/dir/move", &user)?;
    dirs::mov(
        Path::new(&origin).to_owned(),
        Path::new(&destiny).to_owned(),
    )
}

#[post("/dir/del")]
pub async fn dir_del(one: Json<OnePath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &one.path);
    guard::check_dir_access(&path, None, "/dir/del", &user)?;
    dirs::del(Path::new(&path).to_owned())
}

#[post("/file/read")]
pub async fn file_read(
    one: Json<OnePath>,
    req: HttpRequest,
    srv_data: SrvData,
) -> Result<NamedFile, Error> {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &one.path);
    guard::check_dir_access(&path, None, "/file/read", &user)?;
    files::read(Path::new(&path).to_owned())
}

#[post("/file/write")]
pub async fn file_write(rec: Json<PathData>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &rec.path);
    guard::check_dir_access(&path, None, "/file/write", &user)?;
    files::write(Path::new(&path).to_owned(), rec.base64, &rec.data)
}

#[post("/file/append")]
pub async fn file_append(rec: Json<PathData>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &rec.path);
    guard::check_dir_access(&path, None, "/file/append", &user)?;
    files::append(Path::new(&path).to_owned(), rec.base64, &rec.data)
}

#[post("/file/copy")]
pub async fn file_copy(two: Json<TwoPath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let origin = utils::get_absolute(&user, &two.origin);
    let destiny = utils::get_absolute(&user, &two.destiny);
    guard::check_dir_access(&origin, Some(&destiny), "/file/copy", &user)?;
    files::copy(
        Path::new(&origin).to_owned(),
        Path::new(&destiny).to_owned(),
    )
}

#[post("/file/move")]
pub async fn file_move(two: Json<TwoPath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let origin = utils::get_absolute(&user, &two.origin);
    let destiny = utils::get_absolute(&user, &two.destiny);
    guard::check_dir_access(&origin, Some(&destiny), "/file/move", &user)?;
    files::mov(
        Path::new(&origin).to_owned(),
        Path::new(&destiny).to_owned(),
    )
}

#[post("/file/del")]
pub async fn file_del(one: Json<OnePath>, req: HttpRequest, srv_data: SrvData) -> SrvResult {
    let user = guard::get_user(&req, &srv_data);
    if user.is_none() {
        return Err(ErrorForbidden(
            "You don't have access to call this resource.",
        ));
    }
    let user = user.unwrap();
    let path = utils::get_absolute(&user, &one.path);
    guard::check_dir_access(&path, None, "/file/del", &user)?;
    files::del(Path::new(&path).to_owned())
}

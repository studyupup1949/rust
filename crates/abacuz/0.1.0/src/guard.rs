use actix_web::error::{Error, ErrorForbidden};
use actix_web::HttpRequest;

use super::data::Access;
use super::data::User;
use super::SrvData;

pub fn get_user(req: &HttpRequest, srv_data: &SrvData) -> Option<User> {
	if is_debug_local(srv_data, req) {
		let users = &srv_data.users;
		if let Some(root) = users.into_iter().find(|user| user.name == "root") {
			return Some(root.clone());
		}
	}
	get_token_user(req, srv_data)
}

pub fn get_user_or_err(req: &HttpRequest, srv_data: &SrvData) -> Result<User, Error> {
	let user = get_user(req, srv_data);
	if user.is_none() {
		return Err(ErrorForbidden(
			"You don't have access to call this resource.",
		));
	}
	Ok(user.unwrap())
}

pub fn check_dir_access(
	path_ref: &str,
	path_dest: Option<&str>,
	resource: &str,
	user: &User,
) -> Result<(), Error> {
	if check_dir_resource(path_ref, path_dest, resource, user) {
		return Ok(());
	} else {
		return Err(ErrorForbidden(
			"You don't have access to call this resource.",
		));
	}
}

pub fn is_debug_local(srv_data: &SrvData, req: &HttpRequest) -> bool {
	let info = req.connection_info();
	let host = info.host();
	if !host.starts_with("localhost") {
		return false;
	}
	(*srv_data).head.debug
}

fn check_dir_resource(
	path_ref: &str,
	path_dest: Option<&str>,
	resource: &str,
	user: &User,
) -> bool {
	if user.master {
		return true;
	}
	if resource == "/dir/list" {
		return check_dir_read(&user, &path_ref);
	} else if resource == "/dir/new" {
		return check_dir_write(&user, &path_ref);
	} else if resource == "/dir/copy" {
		if let Some(path_dest) = path_dest {
			return check_dir_read(&user, &path_ref) && check_dir_write(&user, &path_dest);
		}
	} else if resource == "/dir/move" {
		if let Some(path_dest) = path_dest {
			return check_dir_write(&user, &path_ref) && check_dir_write(&user, &path_dest);
		}
	} else if resource == "/dir/del" {
		return check_dir_write(&user, &path_ref);
	} else if resource == "/file/read" {
		return check_dir_read(&user, &path_ref);
	} else if resource == "/file/write" {
		return check_dir_write(&user, &path_ref);
	} else if resource == "/file/append" {
		return check_dir_write(&user, &path_ref);
	} else if resource == "/file/upload" {
		return check_dir_write(&user, &path_ref);
	} else if resource == "/file/copy" {
		if let Some(path_dest) = path_dest {
			return check_dir_read(&user, &path_ref) && check_dir_write(&user, &path_dest);
		}
	} else if resource == "/file/move" {
		if let Some(path_dest) = path_dest {
			return check_dir_write(&user, &path_ref) && check_dir_write(&user, &path_dest);
		}
	} else if resource == "/file/del" {
		return check_dir_write(&user, &path_ref);
	} else {
		println!("[DEBUG] We got an unknown resource to check: {}", resource)
	}
	false
}

pub fn check_dir_read(user: &User, check_path: &str) -> bool {
	for user_access in &user.access {
		match user_access {
			Access::DIR { path, write: _ } => {
				if check_path.starts_with(path) {
					return true;
				}
			}
		}
	}
	false
}

pub fn check_dir_write(user: &User, check_path: &str) -> bool {
	for user_access in &user.access {
		if let Access::DIR { path, write } = user_access {
			if check_path.starts_with(path) && *write {
				return true;
			}
		}
	}
	false
}

pub fn get_token_user(req: &HttpRequest, srv_data: &SrvData) -> Option<User> {
	let got_token = get_qinpel_token(req);
	if got_token.is_empty() {
		return None;
	}
	let our_tokens = &srv_data.tokens.read().unwrap();
	let found_auth = our_tokens.get(&got_token);
	if found_auth.is_none() {
		return None;
	}
	let found_auth = found_auth.unwrap();
	return Some(found_auth.user.clone());
}

pub fn get_qinpel_token(req: &HttpRequest) -> String {
	if let Some(token) = req.headers().get("Qinpel-Token") {
		if let Ok(token) = token.to_str() {
			return String::from(token);
		}
	}
	String::new()
}

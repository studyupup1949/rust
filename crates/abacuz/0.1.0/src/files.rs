use actix_files::NamedFile;
use actix_web::error::{Error, ErrorBadRequest};
use actix_web::HttpResponse;
use base64;

use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use super::SrvResult;

pub fn read(path: PathBuf) -> Result<NamedFile, Error> {
	Ok(NamedFile::open(path)?)
}

fn write_data(mut file: File, base64: bool, data: &String) -> Result<(), Error> {
	if base64 {
		let bytes = base64::decode(data);
		if bytes.is_err() {
			return Err(ErrorBadRequest("Could not decode de base 64 data."));
		}
		let bytes = bytes.unwrap();
		file.write_all(&bytes)?;
	} else {
		file.write_all(data.as_bytes())?;
	}
	Ok(())
}

pub fn write(path: PathBuf, base64: bool, data: &String) -> SrvResult {
	let file = OpenOptions::new().create(true).write(true).append(false).open(&path)?;
	write_data(file, base64, data)?;
	Ok(HttpResponse::Ok().body(&format!("Written on: {}", path.display())))
}

pub fn append(path: PathBuf, base64: bool, data: &String) -> SrvResult {
	let file = OpenOptions::new().create(true).write(true).append(true).open(&path)?;
	write_data(file, base64, data)?;
	Ok(HttpResponse::Ok().body(&format!("Appended on: {}", path.display())))
}

pub fn copy(origin: PathBuf, destiny: PathBuf) -> SrvResult {
	if !origin.exists() {
		return Err(ErrorBadRequest("The file origin to copy does not exists."));
	}
	if origin.is_dir() {
		return Err(ErrorBadRequest("The file origin to copy is a directory."));
	}
	fs::copy(&origin, &destiny)?;
	Ok(HttpResponse::Ok().body(format!(
		"File copied from: {} to: {}",
		origin.display(),
		destiny.display()
	)))
}

pub fn mov(origin: PathBuf, destiny: PathBuf) -> SrvResult {
	if !origin.exists() {
		return Err(ErrorBadRequest("The file origin to copy does not exists."));
	}
	if origin.is_dir() {
		return Err(ErrorBadRequest("The file origin to copy is a directory."));
	}
	fs::copy(&origin, &destiny)?;
	fs::remove_file(&origin)?;
	Ok(HttpResponse::Ok().body(format!(
		"File moved from: {} to: {}",
		origin.display(),
		destiny.display()
	)))
}

pub fn del(path: PathBuf) -> SrvResult {
	if !path.exists() {
		return Err(ErrorBadRequest("The file to delete does not exists."));
	}
	if path.is_dir() {
		return Err(ErrorBadRequest("The file to delete is a directory."));
	}
	fs::remove_file(&path)?;
	Ok(HttpResponse::Ok().body(format!("File deleted: {}", path.display())))
}

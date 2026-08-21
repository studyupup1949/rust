/**
 * Aldaron's Device Interface - "build.rs"
 * Copyright 2017 (c) Jeron Lau - Licensed under the GNU GENERAL PUBLIC LICENSE
**/

extern crate gcc;

#[cfg(target_os = "linux")]
fn link() {
	// TODO: Link Statically
	println!("cargo:rustc-link-lib=vulkan1-0-39");
//	println!("cargo:rustc-link-lib=static=vulkan-1");
}

#[cfg(target_os = "windows")]
fn link() {
	// TODO: Link Statically
	println!("cargo:rustc-link-lib=vulkan-1");
//	println!("cargo:rustc-link-lib=static=vulkan-1");
}

fn main() {
	gcc::Config::new().file("src/native/vw.c").flag("-Wall").flag("-Werror").compile("libaldaronvw.a");
	println!("cargo:rustc-link-search=native=src/third-party/");
	println!("cargo:rustc-link-args=-Wl,--subsystem,windows");
	link();
}

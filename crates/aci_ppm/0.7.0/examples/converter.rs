extern crate aci_ppm;
extern crate aci_png;

use std::{ fs::File, io::Write };

fn main() {
	let logo_bar = include_bytes!("logo-bar.png");

	let graphic = aci_png::decode(logo_bar).unwrap();
	let ppm = aci_ppm::encode(graphic.clone());
	let mmd = aci_ppm::encode_mmd(graphic, false);

	println!("MMD is {} bytes", mmd.len());

	let mut ppm_file = File::create("logo-bar.ppm").unwrap();
	let mut mmd_file = File::create("logo-bar.mmd").unwrap();

	ppm_file.write_all(&ppm).unwrap();
	mmd_file.write_all(&mmd).unwrap();

	let verify = aci_ppm::decode_mmd(&mmd).unwrap();
	let verify = aci_png::encode(verify, false);
	let mut verify_file = File::create("verify.png").unwrap();
	verify_file.write_all(&verify).unwrap();
}

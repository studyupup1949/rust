extern crate aci_ppm;
extern crate aci_png;

use std::{ fs::File, io::{ Write, Read, BufReader } };

fn main() {
	use std::env;

	let args: Vec<String> = env::args().collect();

	if args.len() < 3 {
		println!("Usage:\n\t{} out.mmd in1.png in2.png etc.\n", args[0]);
		println!("Not enough arguments!");
		::std::process::exit(-1);
	}

	// Get Output
	let output_file = &args[1];
	// Get Input
	let input_files = &args[2..];

	let mut output = Vec::new();
	for file in input_files {
		let input_file = File::open(file).unwrap();
		let mut buf_reader = BufReader::new(input_file);
		let mut input = vec![];
		buf_reader.read_to_end(&mut input).unwrap();
		let graphic = aci_png::decode(&input).unwrap();
		output.extend(aci_ppm::encode_mmd(graphic, false));
	}

	let mut mmd_file = File::create(output_file).unwrap();
	mmd_file.write_all(&output).unwrap();
}

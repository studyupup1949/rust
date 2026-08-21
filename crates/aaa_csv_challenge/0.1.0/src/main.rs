// mod core;
// mod err;
// mod opt;

// use self::opt::Opt;
// use crate::core::read::{load_csv, write_csv};
// use crate::core::write::replace_column;
// use opt::Opt;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

use aaa_csv_challenge::{
    replace_column, Opt, {load_csv, write_csv},
};

fn main() {
    // println!("Hello, world!");
    let opt = Opt::from_args();
    println!("####-main, opt = {:?}", opt);
    let filename = PathBuf::from(opt.input);

    let csv_data = match load_csv(filename) {
        Ok(data) => data,
        Err(e) => {
            println!("main error: {:?}", e);
            process::exit(1);
        }
    };

    let csv_data = match replace_column(csv_data, &opt.column_name, &opt.replacement) {
        Ok(data) => data,
        Err(e) => {
            println!("main error : {:?}", e);
            process::exit(1);
        }
    };

    let output_file = &opt.output.unwrap_or("output/optput-2.csv".to_string());
    match write_csv(&csv_data, &output_file) {
        Ok(_) => println!("write success!"),
        Err(e) => {
            println!("main error: {:?}", e);
            process::exit(1);
        }
    }
}

//! This example shows how to only process new QSOs in a file
//! This comes in handy when reading e.g. WSJTX log (wsjtx_log.adi) and only processing newly added QSOs

use adif_io::{DeserializeADI, Doc};
use core::time::Duration;
use std::{env, fs};
use std::thread::sleep;
use clap::Parser;
use log::{debug, info};

/// Monitor an ADI file for new QSOs and prints them
/// Initially all available QSOs are printed
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// The ADI file to monitor (e.g. /home/user/.local/share/WSJT-X/wsjtx_log.adi)
    file: String,

    /// The interval to check for new QSOs
    #[arg(short, long, default_value_t = 5)]
    interval: u8,

    /// The amount of QSOs to skip initially
    #[arg(short, long, default_value_t = 0)]
    skip: usize,
}

fn main() {
    let _ = env::var("RUST_LOG").is_err_and(|_| {
        unsafe { env::set_var("RUST_LOG", "info") }
        false
    });
    env_logger::init();

    let args = Args::parse();

    info!("Monitoring ADI file '{}'...", &args.file);

    let mut qso_cnt = args.skip;

    loop {
        debug!("Checking file...");
        let mut doc = Doc::new();
        doc.deserialize_adi(
            fs::read_to_string(&args.file).expect("error reading ADI file: {err}"),
        )
        .expect("could not deserialize from ADI");

        debug!("Skipping {} QSOs...", qso_cnt);
        doc.iter_records()
            .skip(qso_cnt)  // Skip already read QSOs
            .for_each(|qso| println!("{}", qso));
        qso_cnt = doc.iter_records().count();  // Store current QSO count to skip over next time

        sleep(Duration::from_secs(args.interval as u64));
    }
}

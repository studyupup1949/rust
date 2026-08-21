use clap::ArgMatches;
use serde_json::Value;
use std::path::Path;

use std::sync::atomic::Ordering;

static DEFAULT_HOST: &str = "localhost";
static DEFAULT_PORT: u64 = 5490;

#[derive(Debug)]
pub struct Head {
    pub debug: bool,
    pub verbose: bool,
    pub host: String,
    pub port: u64,
}

impl Head {
    pub fn load(args: ArgMatches<'_>) -> Self {
        let mut setup_debug = false;
        let mut setup_verbose = false;
        let mut setup_host = String::from(DEFAULT_HOST);
        let mut setup_port = DEFAULT_PORT;
        let path = Path::new("setup.json");
        if path.exists() {
            let file = std::fs::File::open(path).expect("Setup file exists but could not be open.");
            let setup_file: Value =
                serde_json::from_reader(file).expect("Setup file exists but could not be parsed.");
            match &setup_file["serverDebug"] {
                Value::Bool(server_debug) => {
                    setup_debug = *server_debug;
                }
                _ => {}
            };
            match &setup_file["serverVerbose"] {
                Value::Bool(server_verbose) => {
                    setup_verbose = *server_verbose;
                }
                _ => {}
            };
            match &setup_file["serverHost"] {
                Value::String(server_host) => {
                    setup_host = String::from(server_host);
                }
                _ => {}
            };
            match &setup_file["serverPort"] {
                Value::Number(server_port) => {
                    setup_port = server_port
                        .as_u64()
                        .expect("Could not parse the server port from setup file.");
                }
                _ => {}
            };
        }
        if args.is_present("debug") {
            setup_debug = true;
        }
        if args.is_present("verbose") {
            setup_verbose = true;
        }
        if args.is_present("host") {
            setup_host = String::from(
                args.value_of("host")
                    .expect("Could not read the host argument."),
            );
        }
        if args.is_present("port") {
            setup_port = args
                .value_of("port")
                .expect("Could not read the port argument.")
                .parse()
                .expect("Could not parse the port argument.");
        }
        if setup_debug {
            crate::DEBUG.store(true, Ordering::Relaxed);
        }
        if setup_verbose {
            crate::VERBOSE.store(true, Ordering::Relaxed);
        }
        Head {
            debug: setup_debug,
            verbose: setup_verbose,
            host: setup_host,
            port: setup_port,
        }
    }
}

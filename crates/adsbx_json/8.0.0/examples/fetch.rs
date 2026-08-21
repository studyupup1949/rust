use adsbx_json::v2::Response;
use chrono::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct CliArgs {
    #[structopt(
        long,
        help = "Fetch JSON from this file. Use `-` for stdin.",
        required_unless = "url"
    )]
    file: Option<String>,
    #[structopt(long, help = "Fetch JSON from this URL.", required_unless = "file")]
    url: Option<String>,
}

fn headers_from_creds(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "api-auth",
        HeaderValue::from_str(api_key).expect("Invalid characters in API key"),
    );
    headers
}

fn main() {
    let args = CliArgs::from_args();
    let body = if let Some(url) = args.url {
        let api_key = std::env::var("ADSBX_API_KEY").expect("ADSBX_API_KEY env var not set");
        let client = reqwest::blocking::Client::builder()
            .user_agent("adsbx_json_fetch / 0.0")
            .gzip(true)
            .default_headers(headers_from_creds(api_key.as_str()))
            .build()
            .unwrap();
        let resp = client.get(&url).send().unwrap();
        let resp = resp.error_for_status().unwrap();
        resp.text().unwrap()
    } else if let Some(filepath) = args.file {
        if filepath == "-" {
            let stdin = io::stdin();
            let mut buffered_reader = BufReader::new(stdin);
            let mut contents = String::new();
            buffered_reader.read_to_string(&mut contents).unwrap();
            contents
        } else {
            let file = File::open(filepath).unwrap();
            let mut buffered_reader = BufReader::new(file);
            let mut contents = String::new();
            buffered_reader.read_to_string(&mut contents).unwrap();
            contents
        }
    } else {
        panic!("OW");
    };

    let response = Response::from_str(&body).unwrap();
    println!("now:       {}", Utc::now());
    println!("adsbx.now: {}", response.now);
    println!("cache:     {}", response.cache_time);
    for ac in response.aircraft {
        println!(
            "  {} seen: {:4.1}    seen_pos: {:4.1}",
            ac.hex,
            ac.seen.as_secs_f32(),
            if let Some(d) = ac.seen_pos {
                format!("{}", d.as_secs_f32())
            } else {
                "".to_string()
            }
        );
    }
}

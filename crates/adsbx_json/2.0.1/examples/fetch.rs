use adsbx_json::v2::Response;
use reqwest::header::{HeaderMap, HeaderValue};
use std::str::FromStr;

fn headers_from_creds(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "api-auth",
        HeaderValue::from_str(api_key).expect("Invalid characters in API key"),
    );
    headers
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let api_key = std::env::var("ADSBX_API_KEY").expect("ADSBX_API_KEY env var not set");
    let url = &args[1];
    let client = reqwest::blocking::Client::builder()
        .user_agent("adsbx_json_fetch / 0.0")
        .gzip(true)
        .default_headers(headers_from_creds(api_key.as_str()))
        .build()
        .unwrap();
    let resp = client.get(url).send().unwrap();
    let resp = resp.error_for_status().unwrap();
    let body = resp.text().unwrap();

    let response = Response::from_str(&body).unwrap();
    println!("Got {} aircraft.", response.aircraft.len());

    println!("{:#?}", response);
}

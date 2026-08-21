use super::BlobStorageBase;
use crate::security::{hash_str, SignedV4Base};
use anyhow::{anyhow, Error};
use chrono::Utc;
use reqwest::Url;

pub struct WasabiStorage {
    signer: crate::security::aws::SignedV4Aws,
}

impl WasabiStorage {
    pub fn new(
        region: &'static str,
        access_key: &'static str,
        secret_access_key: &'static str,
    ) -> WasabiStorage {
        let aws = crate::security::aws::SignedV4Aws::new(
            "s3",
            region,
            "s3.wasabisys.com",
            access_key,
            secret_access_key,
        );
        WasabiStorage { signer: aws }
    }
}

impl BlobStorageBase for WasabiStorage {
    fn get(&self, bucket: &str, key: &str) -> Result<Vec<u8>, Error> {
        let now = Utc::now();
        let x_amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let clean_bucket = bucket.replace("/", "");
        let clean_key = match key {
            x if key.starts_with("/") => &x[1..],
            y => y,
        };

        let path = format!("/{}/{}", clean_bucket, clean_key);
        println!("PATH SET TO : {}", path);

        let auth = self
            .signer
            .generate_auth_header("", "GET", path.as_str(), now);
        let url =
            Url::parse(format!("https://s3.wasabisys.com/{}/{}", bucket, key).as_str()).unwrap();

        println!("URL: {}", url.to_string());
        println!("Auth: {}", auth.clone());
        println!("Date: {}", x_amz_date.clone());
        println!("Sha256: {}", hash_str("".to_string()));

        let client = reqwest::blocking::Client::new();
        let req = client
            .get(url)
            .header("Authorization", auth)
            .header("x-amz-date", x_amz_date)
            .header("x-amz-content-sha256", hash_str("".to_string()))
            .build();

        let clean_req = req.unwrap();

        println!("Requst debug: {:?}", clean_req);

        let resp = client.execute(clean_req).unwrap();

        return match resp.status().as_u16() {
            i if i < 300 => {
                let body = resp.bytes().unwrap();
                Ok(body.to_vec())
            }
            _ => {
                let e = format!("Error returned: {}", resp.text().unwrap());
                Err(anyhow!(e))
            }
        };
    }

    fn put(&self, _key: &str, _value: Vec<u8>) -> Result<(), Error> {
        Ok(())
    }
}

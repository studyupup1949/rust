use crate::security::{hash_str, sign, unsigned_payload_hash, SignedV4Base};
use chrono::{DateTime, Utc};

pub struct SignedV4Aws {
    service: &'static str,
    region: &'static str,
    domain: &'static str,
    access_key: &'static str,
    secret_access_key: &'static str,
}

impl SignedV4Aws {
    // create a new instance of the awsv4 signing struct
    pub fn new(
        service: &'static str,
        region: &'static str,
        domain: &'static str,
        access_key: &'static str,
        secret_access_key: &'static str,
    ) -> SignedV4Aws {
        SignedV4Aws {
            service,
            region,
            domain,
            access_key,
            secret_access_key,
        }
    }
}

// Implement the Amazon specific V4 signing process
impl SignedV4Base for SignedV4Aws {
    fn generate_auth_header(
        &self,
        payload: &str,
        method: &str,
        path: &str,
        now: DateTime<Utc>,
    ) -> String {
        let amz_content_256: String = match method {
            "GET" | "HEAD" => hash_str(payload.to_string()),
            _ => unsigned_payload_hash(),
        };

        let x_amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let x_amz_today = now.format("%Y%m%d").to_string();

        // The spec says we should urlencode everything but the `/`
        let encoded_path: String = urlencoding::encode(path);
        let final_encoded_path = encoded_path.replace("%2F", "/");

        // These must be sorted alphabetically
        let canonical_headers = format!(
            "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
            self.domain, amz_content_256, x_amz_date
        );

        let canonical_query = "";

        // These must be alphabetic
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method,
            final_encoded_path,
            canonical_query,
            canonical_headers,
            signed_headers,
            amz_content_256
        );

        let scope = format!(
            "{}/{}/{}/aws4_request",
            x_amz_today, self.region, self.service
        );

        let signed_canonical_request = hash_str(canonical_request);

        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            x_amz_date, scope, signed_canonical_request
        );

        // Generate the signature through the multi-step signing process
        let k_secret = format!("AWS4{}", self.secret_access_key);
        let k_date = sign(k_secret.as_bytes().to_vec(), x_amz_today);
        let k_region = sign(k_date.to_vec(), self.region.to_string());
        let k_service = sign(k_region.to_vec(), self.service.to_string());
        let k_signing = sign(k_service.to_vec(), "aws4_request".to_string());

        // Final signature
        let signature = hex::encode(sign(k_signing.to_vec(), string_to_sign.to_string()));

        // Generate the Authorization header value
        format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, scope, signed_headers, signature
        )
    }
}

/// JAdES B-B test — generates a B-B signature and writes it to a temp file for DSS inspection.
///
/// Run: cargo test --features "soft,jades" --test jades_bb
#[cfg(feature = "jades")]
mod jades_bb_tests {
    use ades::{jades, signer::SoftSigner};

    #[test]
    fn jades_bb_roundtrip() {
        let signer = SoftSigner::generate(2048).expect("key generation failed");
        let data = b"hello from jades B-B test";

        let jws_bytes = jades::sign(data, &signer).expect("jades::sign failed");
        assert!(!jws_bytes.is_empty(), "JWS must not be empty");

        let jws = std::str::from_utf8(&jws_bytes).expect("result must be valid UTF-8");
        assert!(jws.starts_with('{'), "must be a JSON object");
        assert!(jws.contains("\"payload\""), "must contain payload field");
        assert!(
            jws.contains("\"protected\""),
            "must contain protected field"
        );
        assert!(
            jws.contains("\"signature\""),
            "must contain signature field"
        );
        assert!(jws.contains("\"header\""), "must contain header field");

        let tmp = std::env::temp_dir();
        let path = tmp.join("jades_bb_test.json");
        std::fs::write(&path, &jws_bytes).expect("write artifact failed");

        println!("JAdES B-B: {} bytes → {}", jws_bytes.len(), path.display());
        println!("validate: cargo run -p dss-client -- --no-trust sign-jades-bb");
    }
}

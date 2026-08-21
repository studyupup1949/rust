/// JAdES B-T test — generates a B-T signature and writes it to a temp file for DSS inspection.
///
/// Run: cargo test --features "soft,jades,tsp" --test jades_bt -- --ignored
#[cfg(all(feature = "jades", feature = "tsp"))]
mod jades_bt_tests {
    use ades::{jades, signer::SoftSigner, tsp::TspClient};

    #[test]
    #[ignore]
    fn jades_bt_roundtrip() {
        let signer = SoftSigner::generate(2048).expect("key generation failed");
        let tsa = TspClient::new(ades::tsp::client::FREETSA_URL);
        let data = b"hello from jades B-T test";

        let jws_bytes = jades::sign_t(data, &signer, &tsa).expect("jades::sign_t failed");
        assert!(!jws_bytes.is_empty(), "JWS must not be empty");

        let jws = std::str::from_utf8(&jws_bytes).expect("result must be valid UTF-8");
        assert!(
            jws.contains("etsiU"),
            "must contain etsiU in unprotected header"
        );
        assert!(jws.contains("sigTst"), "must contain sigTst");
        assert!(jws.contains("tstTokens"), "must contain tstTokens");

        let tmp = std::env::temp_dir();
        let path = tmp.join("jades_bt_test.json");
        std::fs::write(&path, &jws_bytes).expect("write artifact failed");

        println!("JAdES B-T: {} bytes → {}", jws_bytes.len(), path.display());
        println!("validate: cargo run -p dss-client -- --no-trust sign-jades-t");
    }
}

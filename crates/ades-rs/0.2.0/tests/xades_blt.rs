/// XAdES B-LT test — generates a B-LT signature and writes it to a temp file for DSS inspection.
///
/// With self-signed test certificates (no OCSP URL in AIA), sign_lt silently
/// degrades to B-T. DSS will therefore validate the output as XAdES-BASELINE-T.
///
/// Run: cargo test --features "cades,pades,soft,tsp,ocsp,xades" --test xades_blt -- --ignored
#[cfg(all(feature = "xades", feature = "tsp", feature = "ocsp"))]
mod xades_blt_tests {
    use ades::{ocsp::OcspClient, signer::SoftSigner, tsp::TspClient, xades};

    #[test]
    #[ignore]
    fn xades_blt_roundtrip() {
        let signer = SoftSigner::generate(2048).expect("key generation failed");
        let tsa = TspClient::new(ades::tsp::client::FREETSA_URL);
        let ocsp = OcspClient::new();
        let data = b"hello from xades B-LT test";

        let xml_bytes = xades::sign_lt(data, &signer, &tsa, &ocsp).expect("xades::sign_lt failed");
        assert!(!xml_bytes.is_empty(), "XML must not be empty");

        let xml = std::str::from_utf8(&xml_bytes).expect("result must be valid UTF-8");
        assert!(xml.starts_with("<?xml"), "must start with XML declaration");
        assert!(xml.contains("<ds:Signature"), "must contain ds:Signature");
        assert!(
            xml.contains("<xades:SignatureTimeStamp"),
            "must contain xades:SignatureTimeStamp"
        );
        assert!(
            xml.contains("<xades:UnsignedProperties"),
            "must contain xades:UnsignedProperties"
        );

        let tmp = std::env::temp_dir();
        let path = tmp.join("xades_blt_test.xml");
        std::fs::write(&path, &xml_bytes).expect("write artifact failed");

        println!("XAdES B-LT: {} bytes → {}", xml_bytes.len(), path.display());
        println!("validate: cargo run -p dss-client -- --no-trust sign-xades-lt");
    }
}

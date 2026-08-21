accio::accio_emit! {
    testEnum{
        EnumValOne(String),
    }
    testMatch {
        TestEnum::EnumValOne(s) => {
            println!("val1: '{}'", s);
        },
    }
    testParse {
        if let Some(s) = s.strip_prefix("s_") {
            return Some(TestEnum::EnumValOne(s.to_string()));
        }
    }
}

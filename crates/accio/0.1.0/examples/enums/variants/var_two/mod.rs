accio::accio_emit! {
    testEnum {
        EnumValTwo(i32),
    }
    testMatch {
        TestEnum::EnumValTwo(v) => {
            println!("val2: #{}", v);
        },
    }
    testParse {
        if let Ok(i) = s.parse::<i32>() {
            return Some(TestEnum::EnumValTwo(i));
        }
    }
}

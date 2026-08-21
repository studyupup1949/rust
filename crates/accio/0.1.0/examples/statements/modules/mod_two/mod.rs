accio::accio_emit! {
    secondScope {
        println!("setting value as 5 in the second scope");
        val = 5;
    }
}

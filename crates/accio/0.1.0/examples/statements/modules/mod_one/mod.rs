// did you realize these modules are not even imported?
accio::accio_emit! {
    firstScope {
        println!("setting value as 3 in the first scope");
        val = 3;
    }
}

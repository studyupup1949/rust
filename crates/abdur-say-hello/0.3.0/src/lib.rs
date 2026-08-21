#[cfg(feature = "hello")]
pub fn say_hello(name: &str) -> String {
    format!("Hello {}!", name)
}

#[cfg(feature = "hello")]
pub fn say_hello_to_everyone() -> String {
    format!("Hello everyone!")
}

#[cfg(feature = "bye")]
pub fn say_goodbye(name: &str ) -> String {
    format!("Goodbye {}!", name)
}

#[cfg(feature = "bye")]
pub fn say_goodbye_to_everyone() -> String {
    format!("Goodbye everyone!")
}

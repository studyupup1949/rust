#[cfg(feature= "hello")]
pub fn say_hello(name: &str)->String {
    format!("Hello, {}!", name)
}

#[cfg(feature= "hello")]
pub fn say_hello_everyone()->String {
  "Hello Everyone!".to_string()
}

#[cfg(feature= "bye")]
pub fn say_goodbye(name: &str)->String {
    format!("Goodbye, {}!", name)
}

#[cfg(feature= "bye")]
pub fn say_goodbye_everyone()->String {
  "Goodbye Everyone!".to_string()
}
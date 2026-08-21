pub fn say_hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn say_goodbye(name: &str) -> String {
    format!("Good bye, {}!", name)
}

pub fn say_hello_eveyone() -> String {
    "Hello, everyone!".to_string()
}

pub fn say_goodbye_everyone() -> String {
    "Good bye, everyone!".to_string()
}

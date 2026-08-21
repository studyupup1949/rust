//! Abracadabra: yrs crdt server framework

/// A simple function that returns a greeting.
pub fn greeting() -> &'static str {
    "Welcome to Abracadabra!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(greeting(), "Welcome to Abracadabra!");
    }
}

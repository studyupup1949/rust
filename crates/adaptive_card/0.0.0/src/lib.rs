

pub struct AdaptiveCard {}

impl AdaptiveCard {
    pub fn new() -> Self {
        Self {}
    }
    pub fn hi(&self) -> () {
        println!("hello");
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

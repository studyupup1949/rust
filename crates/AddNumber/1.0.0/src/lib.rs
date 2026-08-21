pub fn add(number_1: u64, number_2: u64) -> u64 {
    number_1 + number_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

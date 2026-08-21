pub mod level;

#[cfg(test)]
mod tests {
    use crate::level;
    use std::fs;

    #[test]
    fn load() {
        let file = fs::read_to_string("test_files/level.adofai").unwrap();

        let lvl = level::load(&file);

        println!("{:?}", lvl)
    }
}

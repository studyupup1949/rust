#[cfg(test)]
mod tests {
    use abs_file_macro::abs_file;
    use std::fs;

    #[test]
    fn test_absolute_path() {
        let path = abs_file!();
        assert!(
            path.is_absolute(),
            "abs_file!() should return an absolute path, got: {:?}",
            path
        );
    }

    #[test]
    fn test_path_is_valid_file() {
        let path = abs_file!();
        assert!(
            fs::metadata(&path).is_ok(),
            "abs_file!() should point to a real file, but {:?} does not exist",
            path
        );
    }

    #[test]
    fn test_path_contains_file_name() {
        let path = abs_file!();
        let file_name = "abs_file_tests.rs"; // This test file itself
        assert!(
            path.ends_with(file_name),
            "Expected path to end with `{}`, but got: {:?}",
            file_name,
            path
        );
    }

    mod nested {
        use super::*;

        #[test]
        fn test_nested_module_path_is_valid() {
            let path = abs_file!();
            assert!(
                fs::metadata(&path).is_ok(),
                "abs_file!() inside a nested module should still point to a real file, but {:?} does not exist",
                path
            );
        }
    }
}

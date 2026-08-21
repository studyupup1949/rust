// 集成测试
#[cfg(test)]
mod test {
    use aaa_csv_challenge::{
        replace_column, Opt, {load_csv, write_csv},
    };
    use std::path::PathBuf;

    #[test]
    fn test_aaa_csv_challenge() {
        let filename = PathBuf::from("./input/challenge.csv");
        let csv_data = load_csv(filename);
        assert!(csv_data.is_ok());

        let csv_data = replace_column(csv_data.unwrap(), "City", "US");
        assert!(csv_data.is_ok());

        let output_file = write_csv(&csv_data.unwrap(), "./output/output-test.csv");
        assert!(output_file.is_ok());

        println!("####-integration_test is OK!");
    }
}

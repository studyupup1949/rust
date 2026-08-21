use super::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Bytes, Read, Write};
use std::path::PathBuf;

pub fn load_csv(csv_file: PathBuf) -> Result<String, Error> {
    let file = read(csv_file)?;
    Ok(file)
}

// 文档测试
// 如果将ignore去掉，在执行 cargo test 命令时，文档注释中的代码会被当成Rust代码执行；
// 如果去掉 ``` 语法，则生成的代码无法高亮显示；
/// # Usage:
/// ```ignore
/// let filename = PathBuf::form("./input/challenge.csv");
/// let csv_data = load_csv(filename).unwrap();
/// let modified_data = replace_column(csv_data, "City", "Beijing").unwrap();
/// let output_file = write_csv(&modified_data, "./output/output_test.csv");
/// assert!(output_file.is_ok());
/// ````
///
pub fn write_csv(csv_data: &str, filename: &str) -> Result<(), Error> {
    write(csv_data, filename)?;
    Ok(())
}

fn read(path: PathBuf) -> Result<String, Error> {
    let mut buffer = String::new();
    let mut file = open(path)?;
    file.read_to_string(&mut buffer)?;
    if buffer.is_empty() {
        return Err("input file missing")?;
    }
    Ok(buffer)
}

fn open(path: PathBuf) -> Result<File, Error> {
    let file = File::open(path)?;
    Ok(file)
}

fn write(data: &str, filename: &str) -> Result<(), Error> {
    let mut buffer = File::create(filename)?;
    buffer.write_all(data.as_bytes())?;
    Ok(())
}

//单元测试
#[cfg(test)]
mod test {
    use crate::core::read::load_csv;
    use std::path::PathBuf;

    #[test]
    // #[ignore]
    fn test_load_csv() {
        let filepath = PathBuf::from("./input/challenge.csv");
        let csv_data = load_csv(filepath);
        assert!(csv_data.is_ok())
    }
}

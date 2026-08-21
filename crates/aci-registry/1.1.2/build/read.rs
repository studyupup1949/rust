use std::{fs::File, io::ErrorKind, path::Path};

use serde::de::DeserializeOwned;

pub fn read<S: AsRef<Path> + ?Sized, T: DeserializeOwned>(path: &S) -> std::io::Result<Vec<T>> {
    let path = path.as_ref();
    println!("cargo:rerun-if-changed={}", path.display());
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    reader
        .deserialize()
        .map(|x| {
            x.map_err(|e| match e.into_kind() {
                csv::ErrorKind::Io(err) => err,
                csv::ErrorKind::Deserialize { err, .. } => {
                    std::io::Error::new(ErrorKind::InvalidData, err)
                }
                csv::ErrorKind::UnequalLengths {
                    pos,
                    expected_len,
                    len,
                } => {
                    use core::fmt::Write;
                    let mut str = String::new();
                    if let Some(pos) = pos {
                        write!(str, "Line {}: ", pos.line()).unwrap();
                    }

                    write!(
                        str,
                        "Invalid length {}, expected length {}",
                        len, expected_len
                    )
                    .unwrap();
                    std::io::Error::new(ErrorKind::InvalidData, str)
                }
                e => std::io::Error::new(ErrorKind::Other, format!("{:?}", e)),
            })
        })
        .collect::<std::io::Result<_>>()
}

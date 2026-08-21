#[cfg(doctest)]
doc_comment::doctest!("../README.md");

/// A macro that returns the absolute file path of the Rust source file
/// in which it is invoked.
///
/// This macro ensures that the correct absolute path is resolved, even
/// when used within a Cargo workspace or a nested crate. It prevents
/// issues with duplicated path segments by properly handling the crate
/// root.
///
/// # How It Works
/// - `file!()` provides the relative path of the current source file.
/// - `env!("CARGO_MANIFEST_DIR")` provides the absolute path of the crate root.
/// - The macro joins these paths while ensuring no duplicate segments.
///
/// # Example
/// ```
/// use abs_file_macro::abs_file;
///
/// fn main() {
///     let path = abs_file!();
///     println!("Absolute file path: {:?}", path);
/// }
/// ```
///
/// # Edge Cases
/// - If the `file!()` output already includes the crate name, it is stripped.
/// - Ensures that the output is always an **absolute** path.
/// - Does not depend on runtime conditions.
///
/// # Testing
/// This macro is tested in two ways:
/// 1. **As part of the full workspace**
///    ```sh
///    cargo test --workspace
///    ```
/// 2. **Within an individual crate**
///    ```sh
///    cd test-workspace-1
///    cargo test
///    ```
///
/// This ensures correctness across different usage scenarios.
#[macro_export]
macro_rules! abs_file {
    () => {{
        use std::path::{Path, PathBuf};

        const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
        const FILE_PATH: &str = file!(); // Expands where the macro is used

        let manifest_dir = Path::new(MANIFEST_DIR);
        let file_path = Path::new(FILE_PATH);

        let crate_name = manifest_dir.file_name().unwrap();

        let relative_file_path = if file_path.starts_with(crate_name) {
            file_path.strip_prefix(crate_name).unwrap()
        } else {
            file_path
        };

        PathBuf::from(MANIFEST_DIR).join(relative_file_path)
    }};
}

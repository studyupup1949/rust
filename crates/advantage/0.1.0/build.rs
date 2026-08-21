#[cfg(feature = "ffi")]
extern crate cbindgen;

#[cfg(feature = "ffi")]
fn generate_ffi_header() {
    // Generate FFI header
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or(crate_dir.clone() + "/target");
    match cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_parse_expand(&["advantage"])
        .with_parse_expand_features(&["ffi"])
        .with_include_guard("_ADV_FFI_H")
        .rename_item("AContext", "adv_context")
        .rename_item("Tape", "adv_tape")
        .rename_item("ADouble", "adv_double")
        .generate()
    {
        Ok(ffi_header) => {
            ffi_header.write_to_file(target_dir + "/adv_ffi.h");
        }
        Err(err) => {
            println!("Failed to create FFI header: {:?}", err);
        }
    }
}

fn main() {
    #[cfg(feature = "ffi")]
    generate_ffi_header();
}

#[cfg(feature = "csharp")]
fn generate_csharp_bindings() {
    use csbindgen::Builder;
    use std::path::PathBuf;

    let workspace_dir = std::env::var("CARGO_WORKSPACE_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .map(|d| d.parent().expect("workspace root").to_path_buf())
        })
        .expect("CARGO_WORKSPACE_DIR or CARGO_MANIFEST_DIR");
    let ffi_rs = workspace_dir.join("aam-core").join("src").join("ffi.rs");
    let output = workspace_dir
        .join("bindings")
        .join("csharp")
        .join("src")
        .join("AamNative.cs");

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create bindings output directory");
    }

    Builder::default()
        .input_extern_file(ffi_rs.to_str().expect("valid UTF-8 path"))
        .csharp_dll_name("aam_rs")
        .csharp_class_name("AamNative")
        .csharp_namespace("AamCsharp")
        .csharp_use_function_pointer(false)
        .generate_csharp_file(output.to_str().expect("valid UTF-8 path"))
        .expect("Failed to generate C# bindings");
}

fn main() {
    #[cfg(feature = "csharp")]
    {
        generate_csharp_bindings();
    }
}

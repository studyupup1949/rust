#[cfg(feature = "csharp")]
fn generate_csharp_bindings() {
    use csbindgen::Builder;

    Builder::default()
        .input_extern_file("src/ffi.rs")
        .csharp_dll_name("aam_rs")
        .csharp_class_name("AamNative")
        .csharp_namespace("AamCsharp")
        .csharp_use_function_pointer(false)
        .generate_csharp_file("bindings/csharp/src/AamNative.cs")
        .expect("Failed to generate C# bindings");
}

fn main() {
    #[cfg(feature = "csharp")]
    {
        generate_csharp_bindings();
    }
}

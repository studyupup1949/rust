const UNDERLYING_CPP_TYPES : &'static [&'static str] = &[
    // This is ordered roughly by expected likelyhood for performance reasons.

    // Many/most compilers default to `int` for ≈everything, so it's first.
    "int",          "unsigned int",

    // Smaller integers are semi-common for e.g. embedded
    "short",        "unsigned short",
    "signed char",  "unsigned char",
    "char",

    // MSVC truncates 64-bit values to 32-bit, but clang doesn't.
    "long",         "unsigned long",
    "long long",    "unsigned long long",

    // 128-bit is ≈unavailable.
    // MSVC fails to compile 128-bit values.
    // Clang truncates 128-bit values to 64-bit.
];

fn main() {
    let i8_min   = format!("{}",   i8::MIN);
    let i16_min  = format!("{}",  i16::MIN);
    let i32_min  = format!("{}",  i32::MIN);
    let i64_min  = format!("{}",  i64::MIN);
    let i128_min = format!("{}", i128::MIN);

    println!("cargo::rustc-check-cfg=cfg(nope)");

    for (rusty,         min,                                  max) in [
        ("i8",   &*  i8_min,                               "0x7F"),
        ("i16",  &* i16_min,                             "0x7FFF"),
        ("i32",  &* i32_min,                         "0x7FFFFFFF"),
        ("i64",  &* i64_min,                 "0x7FFFFFFFFFFFFFFF"),
        ("i128", &*i128_min, "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
        ("u7",          "0",                               "0x7F"),
        ("u8",          "0",                               "0xFF"),
        ("u15",         "0",                             "0x7FFF"),
        ("u16",         "0",                             "0xFFFF"),
        ("u31",         "0",                         "0x7FFFFFFF"),
        ("u32",         "0",                         "0xFFFFFFFF"),
        ("u63",         "0",                 "0x7FFFFFFFFFFFFFFF"),
        ("u64",         "0",                 "0xFFFFFFFFFFFFFFFF"),
        ("u127",        "0", "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"), // MSVC fails to compile 128-bit stuff.  Clang truncates 128-bit stuff to 64-bit.
        ("u128",        "0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"), // MSVC fails to compile 128-bit stuff.  Clang truncates 128-bit stuff to 64-bit.
    ].iter().copied() {
        print!("cargo::rustc-check-cfg=cfg(c_enum_{rusty}, values(\"unknown\"");
        for cpp_ty in UNDERLYING_CPP_TYPES.iter().copied() { print!(", {cpp_ty:?}"); }
        println!("))");

        let mut unknown = true;
        'underlying_cpp_type: for underlying_cpp_type in UNDERLYING_CPP_TYPES.iter().copied() {
            let compiled = cc::Build::new()
                .file("src/test-enum-type.cpp")
                .cpp(true)
                .define("TEST_ENUM_MIN",  Some(min))
                .define("TEST_ENUM_MAX",  Some(&*format!("{max}")))
                .define("TEST_ENUM_TYPE", Some(underlying_cpp_type))
                .cargo_metadata(false)
                .cargo_warnings(false)
                .cargo_debug(false)
                .cargo_output(false)
                .try_compile_intermediates()
                .is_ok();
            if compiled {
                println!("cargo::rustc-cfg=c_enum_{rusty}=\"{underlying_cpp_type}\"");
                unknown = false;
                break 'underlying_cpp_type;
            }
        }
        if unknown {
            println!("cargo::rustc-cfg=c_enum_{rusty}=\"unknown\"");
        }
    }
}

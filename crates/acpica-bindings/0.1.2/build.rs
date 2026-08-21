use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    path::{Path, PathBuf},
};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir_path = PathBuf::from(out_dir.clone());

    let acpica_dir = find_source_dir(out_dir_path.clone());

    compile(&acpica_dir);
}

use flate2::read::GzDecoder;
use tar::Archive;

fn find_source_dir(out_dir: PathBuf) -> PathBuf {
    // Find source directory already extracted by a previous compilation
    let dirs =
        fs::read_dir(out_dir.clone()).expect("Should have been able to read files in OUT_DIR");

    for dir in dirs {
        let entry = dir.expect("Should have been able to get info about dir entry");

        // Don't use a file as the source directory
        if !entry
            .file_type()
            .expect("Should have been able to get file type")
            .is_dir()
        {
            continue;
        }

        if entry
            .file_name()
            .into_string()
            .unwrap_or(String::new())
            .starts_with("acpica-")
        {
            return entry.path();
        }
    }

    // If the source is not already extracted, extract it

    // Find the tarball
    let mut dirs = fs::read_dir(".").expect("Should have been able to read files in crate root");

    let source_tarball_path = dirs.find_map(|dir| {
        let dir = dir.expect("Should have been able to get info about dir entry");
        let name = dir.file_name().into_string().expect("Should have been able to get file name of dir entry");
        let is_dir = dir.file_type().expect("Should have been able to get file type").is_dir();

        if !is_dir && name.starts_with("acpica-") && name.ends_with(".tar.gz") {
            Some(name)
        } else {
            None
        }
    }).expect("No source tarball: There should be a file in the crate root starting with 'acpica-' and ending with '.tar.gz'");

    // Extract the tarball
    let tarball = File::open(source_tarball_path.clone())
        .expect("Should have been able to open tarball directory");
    let tar = GzDecoder::new(tarball);
    let mut archive = Archive::new(tar);

    for e in archive.entries().expect("Could not parse archive") {
        let mut e = e.expect("Should have been able to load entry from archive");

        let mut path = out_dir.clone();
        path.push(
            e.path()
                .expect("Should have been able to get path of entry in archive"),
        );

        e.unpack(path)
            .expect("Should have been able to unpack entry");
    }

    let mut dir_path = out_dir.clone();
    dir_path.push(source_tarball_path.strip_suffix(".tar.gz").unwrap());

    // Change the source files to fit the specific use case
    update_source(dir_path.clone());

    dir_path
}

/// The ACPICA sources link to the system libc by default.
/// This is an issue for rust OS projects, which may not link to libc at all.
/// Thankfully, by removing definitions of the macros `ACPI_USE_SYSTEM_CLIBRARY` and `ACPI_USE_STANDARD_HEADERS`,
/// ACPICA will link to its own libc functions instead.
///
/// This function comments out all such definitions.
fn update_source(acpica_dir: PathBuf) {
    // Copy acrust.h to include/platfom
    let mut acrust_path = acpica_dir.clone();
    acrust_path.push("source/include/platform/acrust.h");

    #[allow(unused_mut)] // This needs to be mutable for the non-builtin-cache code to work, but would raise a warning otherwise
    let mut acrust_text = fs::read_to_string("./acrust.h")
        .expect("Should have been able to read 'acrust.h'. There should be a file in the crate root with the name 'acrust.h'");

    #[cfg(not(feature = "builtin_cache"))]
    {
        acrust_text = acrust_text.replace("#define ACPI_CACHE_T", "// #define ACPI_CACHE_T");
        acrust_text = acrust_text.replace(
            "#define ACPI_USE_LOCAL_CACHE",
            "// #define ACPI_USE_LOCAL_CACHE",
        );
    }

    fs::write(acrust_path, acrust_text).expect("Should have been able to write to 'acrust.h': There should be a folder inside the source directory called source/include/platform");

    // Replace
    let mut acenv_path = acpica_dir.clone();
    acenv_path.push("source/include/platform/acenv.h");

    let env_text =
        fs::read_to_string(acenv_path.clone()).expect("Should have been able to read 'acenv.h'");

    let (pre_platform_includes, after_platform_includes) = {
        let (pre_platform_includes, rest) = env_text
            .split_once("#if defined(_LINUX)")
            .expect("acenv.h should have contained section of platform-specific includes. This is currently detected using the string '#if defined(_LINUX)'.");
        let (_, post_platform_includes) = rest
            .split_once("#endif")
            .expect("The platform-specific headers should have ended in '#endif'");

        (pre_platform_includes, post_platform_includes)
    };

    let new_env_text =
        pre_platform_includes.to_string() + r#"#include "acrust.h""# + after_platform_includes;

    std::fs::write(acenv_path, new_env_text.as_bytes())
        .expect("Should have been abl to write 'acenv.h'");
}

/// Compiles the code using the [`cc`] crate
fn compile(acpica_dir: &Path) {
    let mut compilation = cc::Build::new();

    // ACPICA miscompiles if compiled with -O2, so limit the optimisation level to -O1
    // TODO: debug these miscompilations? 
    let opt_level = env::var("OPT_LEVEL")
        .unwrap()
        .parse::<u32>()
        .unwrap()
        .min(1);

    compilation
        .warnings(false) // Otherwise lots of annoying warnings are printed
        .include(acpica_dir.to_str().unwrap().to_string() + "/source/include") // Add `source/include` as an include directory
        .define("ACPI_DEBUG_OUTPUT", None) // Set the ACPI_DEBUG_OUTPUT symbol
        .flag("-fno-stack-protector")
        .opt_level(opt_level);

    let mut components_path = acpica_dir.to_path_buf();
    components_path.push("source/components");

    // Only the 'components' directory is needed for the kernel subsystem - the other directories are for cli utils
    let components = fs::read_dir(components_path)
        .expect("Source directory should contain sub-directory '/source/components'");

    for component_dir in components {
        let component_dir =
            component_dir.expect("Should have been able to get info about dir entry");

        // TODO: These might be nice to have
        // Exclude the debugger and disassembler dirs because they give 'undefined type' errors
        if [OsString::from("debugger"), OsString::from("disassembler")]
            .contains(&component_dir.file_name())
        {
            continue;
        }

        for c_file in fs::read_dir(component_dir.path())
            .expect("Should have been able to read component directory")
        {
            let c_file = c_file.expect("Should have been able to get info about dir entry");

            compilation.file(c_file.path());
        }
    }

    compilation.compile("acpica")
}

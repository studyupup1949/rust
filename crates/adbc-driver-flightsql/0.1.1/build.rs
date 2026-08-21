use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use tempfile::tempdir;

const DEFAULT_VERSION: &str = "1.8.0";
const DEFAULT_CHANNEL: &str = "https://conda.anaconda.org/conda-forge";
const PACKAGE_NAME: &str = "libadbc-driver-flightsql";

#[derive(Clone, Copy)]
struct PackageVariant {
    conda_platform: &'static str,
    lib_filename: &'static str,
    default_build: &'static str,
}

impl PackageVariant {
    const fn new(
        conda_platform: &'static str,
        lib_filename: &'static str,
        default_build: &'static str,
    ) -> Self {
        Self {
            conda_platform,
            lib_filename,
            default_build,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::var("TARGET")?;
    let variant = variant_for_target(&target).ok_or_else(|| {
        io::Error::other(format!(
            "Unsupported target '{target}' for ADBC FlightSQL driver"
        ))
    })?;

    let version = env::var("ADBC_FLIGHTSQL_VERSION").unwrap_or_else(|_| DEFAULT_VERSION.to_owned());
    let build =
        env::var("ADBC_FLIGHTSQL_BUILD").unwrap_or_else(|_| variant.default_build.to_owned());
    let channel = env::var("ADBC_FLIGHTSQL_CHANNEL").unwrap_or_else(|_| DEFAULT_CHANNEL.to_owned());

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let lib_path = if let Ok(custom_path) = env::var("ADBC_FLIGHTSQL_LIB_PATH") {
        let custom_path_buf = PathBuf::from(custom_path);
        if custom_path_buf.is_dir() {
            custom_path_buf.join(variant.lib_filename)
        } else {
            custom_path_buf
        }
    } else {
        out_dir.join(variant.lib_filename)
    };

    if lib_path.exists() {
        if fs::metadata(&lib_path)?.len() == 0 {
            fs::remove_file(&lib_path)?;
        } else {
            apply_env_exports(&lib_path, &version);
            return Ok(());
        }
    }

    if !lib_path.exists() {
        let url = format!(
            "{channel}/{platform}/{package}-{version}-{build}.conda",
            channel = channel.trim_end_matches('/'),
            platform = variant.conda_platform,
            package = PACKAGE_NAME,
            version = version,
            build = build
        );

        println!("cargo:warning=Downloading ADBC FlightSQL driver from {url}");

        let archive = download_conda_package(&url)?;
        extract_library(&archive, variant.lib_filename, &lib_path)?;
    }

    apply_env_exports(&lib_path, &version);

    Ok(())
}

fn apply_env_exports(lib_path: &Path, version: &str) {
    println!(
        "cargo:rustc-env=ADBC_FLIGHTSQL_LIB_PATH={}",
        lib_path.display()
    );
    println!("cargo:rustc-env=ADBC_FLIGHTSQL_LIB_VERSION={}", version);
    println!("cargo:rerun-if-env-changed=ADBC_FLIGHTSQL_VERSION");
    println!("cargo:rerun-if-env-changed=ADBC_FLIGHTSQL_BUILD");
    println!("cargo:rerun-if-env-changed=ADBC_FLIGHTSQL_CHANNEL");
    println!("cargo:rerun-if-changed=build.rs");
}

fn variant_for_target(target: &str) -> Option<PackageVariant> {
    match target {
        "x86_64-unknown-linux-gnu" => Some(PackageVariant::new(
            "linux-64",
            "libadbc_driver_flightsql.so",
            "h57b9e7f_1",
        )),
        "x86_64-apple-darwin" => Some(PackageVariant::new(
            "osx-64",
            "libadbc_driver_flightsql.dylib",
            "h4135a9e_1",
        )),
        "aarch64-apple-darwin" => Some(PackageVariant::new(
            "osx-arm64",
            "libadbc_driver_flightsql.dylib",
            "hbbbe3c2_1",
        )),
        "x86_64-pc-windows-msvc" => Some(PackageVariant::new(
            "win-64",
            "adbc_driver_flightsql.dll",
            "h57b9e7f_1",
        )),
        _ => None,
    }
}

fn download_conda_package(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        return Err(io::Error::other(format!(
            "Failed to download driver: HTTP {}",
            response.status()
        ))
        .into());
    }

    Ok(response.bytes()?.to_vec())
}

fn extract_library(
    archive: &[u8],
    lib_filename: &str,
    dest: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cursor = std::io::Cursor::new(archive);
    let mut zip = zip::ZipArchive::new(cursor)?;

    let mut pkg_name = None;
    let mut pkg_bytes = Vec::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_owned();

        if name.starts_with("pkg-") && (name.ends_with(".tar.zst") || name.ends_with(".tar.bz2")) {
            pkg_bytes.clear();
            entry.read_to_end(&mut pkg_bytes)?;
            pkg_name = Some(name);
            break;
        }
    }

    let pkg_name = pkg_name.ok_or_else(|| {
        io::Error::other("Failed to locate pkg-*.tar archive inside .conda package")
    })?;

    let tar_reader: Box<dyn Read> = if pkg_name.ends_with(".tar.zst") {
        Box::new(zstd::stream::read::Decoder::new(&pkg_bytes[..])?)
    } else {
        Box::new(bzip2::read::BzDecoder::new(&pkg_bytes[..]))
    };

    let temp_dir = tempdir()?;
    let mut archive = tar::Archive::new(tar_reader);
    archive.unpack(temp_dir.path())?;

    let lib_dir = temp_dir.path().join("lib");
    let source_path = find_library_file(&lib_dir, lib_filename)?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&source_path, dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&source_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest, perms)?;
    }

    #[cfg(not(unix))]
    fs::set_permissions(dest, fs::metadata(&source_path)?.permissions())?;

    println!(
        "cargo:warning=Copied ADBC FlightSQL driver from {}",
        source_path.display()
    );

    Ok(())
}

fn find_library_file(
    lib_dir: &Path,
    lib_filename: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if !lib_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Package did not contain lib/ directory at {}",
                lib_dir.display()
            ),
        )
        .into());
    }

    let exact = lib_dir.join(lib_filename);
    if exact.exists() {
        return Ok(fs::canonicalize(exact)?);
    }

    let (base, ext) = split_library_name(lib_filename);
    let mut fallback: Option<PathBuf> = None;

    for entry in fs::read_dir(lib_dir)? {
        let path = entry?.path();
        let meta = fs::symlink_metadata(&path)?;
        if !meta.is_file() && !meta.file_type().is_symlink() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !name.starts_with(&base) {
            continue;
        }

        if extension_matches(name, ext.as_deref()) {
            fallback = Some(fs::canonicalize(&path)?);
            if meta.is_file() {
                break;
            }
        }
    }

    fallback.ok_or_else(|| {
        io::Error::other(format!("Failed to find {lib_filename} inside pkg archive")).into()
    })
}

fn split_library_name(name: &str) -> (String, Option<String>) {
    match name.rsplit_once('.') {
        Some((base, ext)) => (base.to_owned(), Some(ext.to_owned())),
        None => (name.to_owned(), None),
    }
}

fn extension_matches(candidate: &str, ext: Option<&str>) -> bool {
    match ext {
        None => true,
        Some("so") => candidate.contains(".so"),
        Some("dylib") => candidate.contains(".dylib"),
        Some(ext) => candidate.ends_with(&format!(".{ext}")),
    }
}

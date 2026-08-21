use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

const DEFAULT_VERSION: &str = "1.9.0";
const PACKAGE_NAME: &str = "adbc-driver-flightsql";
const PYPI_BASE: &str = "https://pypi.org/pypi";

#[derive(Clone, Copy)]
struct PackageVariant {
    wheel_suffix: &'static str,
    lib_filename: &'static str,
}

impl PackageVariant {
    const fn new(wheel_suffix: &'static str, lib_filename: &'static str) -> Self {
        Self {
            wheel_suffix,
            lib_filename,
        }
    }

    fn wheel_filename(&self, version: &str) -> String {
        format!("adbc_driver_flightsql-{version}-{}", self.wheel_suffix)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::var("TARGET")?;
    let variant = variant_for_target(&target).ok_or_else(|| {
        io::Error::other(format!(
            "Unsupported target '{target}' for ADBC FlightSQL driver"
        ))
    })?;

    let version = env::var("ADBC_FLIGHTSQL_VERSION").unwrap_or_else(|_| DEFAULT_VERSION.into());

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let lib_path = resolve_output_path(&variant, &out_dir)?;

    if lib_path.exists() {
        if fs::metadata(&lib_path)?.len() == 0 {
            fs::remove_file(&lib_path)?;
        } else {
            apply_env_exports(&lib_path, &version);
            return Ok(());
        }
    }

    let wheel_filename = variant.wheel_filename(&version);
    let release_meta = fetch_release_metadata(&version)?;
    let file_meta = release_meta
        .urls
        .iter()
        .find(|file| file.filename == wheel_filename)
        .ok_or_else(|| {
            io::Error::other(format!(
                "PyPI release {version} missing wheel {wheel_filename}"
            ))
        })?;

    println!(
        "cargo:warning=Downloading FlightSQL wheel {} ({})",
        file_meta.filename, file_meta.url
    );

    let wheel_bytes = download_wheel(&file_meta.url)?;

    if let Some(expected) = file_meta.digests.sha256.as_deref() {
        verify_sha256(&wheel_bytes, expected)?;
    }

    extract_library_from_wheel(&wheel_bytes, variant.lib_filename, &lib_path)?;
    apply_env_exports(&lib_path, &version);

    Ok(())
}

fn resolve_output_path(
    variant: &PackageVariant,
    out_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(custom_path) = env::var("ADBC_FLIGHTSQL_LIB_PATH") {
        let custom = PathBuf::from(custom_path);
        if custom.is_dir() {
            Ok(custom.join(variant.lib_filename))
        } else {
            Ok(custom)
        }
    } else {
        Ok(out_dir.join(variant.lib_filename))
    }
}

fn apply_env_exports(lib_path: &Path, version: &str) {
    println!(
        "cargo:rustc-env=ADBC_FLIGHTSQL_LIB_PATH={}",
        lib_path.display()
    );
    println!("cargo:rustc-env=ADBC_FLIGHTSQL_LIB_VERSION={version}");
    println!("cargo:rerun-if-env-changed=ADBC_FLIGHTSQL_VERSION");
    println!("cargo:rerun-if-env-changed=ADBC_FLIGHTSQL_LIB_PATH");
    println!("cargo:rerun-if-changed=build.rs");
}

fn variant_for_target(target: &str) -> Option<PackageVariant> {
    match target {
        "x86_64-unknown-linux-gnu" => Some(PackageVariant::new(
            "py3-none-manylinux1_x86_64.manylinux2014_x86_64.manylinux_2_17_x86_64.manylinux_2_5_x86_64.whl",
            "libadbc_driver_flightsql.so",
        )),
        "aarch64-unknown-linux-gnu" => Some(PackageVariant::new(
            "py3-none-manylinux2014_aarch64.manylinux_2_17_aarch64.whl",
            "libadbc_driver_flightsql.so",
        )),
        "x86_64-apple-darwin" => Some(PackageVariant::new(
            "py3-none-macosx_10_15_x86_64.whl",
            "libadbc_driver_flightsql.so",
        )),
        "aarch64-apple-darwin" => Some(PackageVariant::new(
            "py3-none-macosx_11_0_arm64.whl",
            "libadbc_driver_flightsql.so",
        )),
        "x86_64-pc-windows-msvc" => Some(PackageVariant::new(
            "py3-none-win_amd64.whl",
            "adbc_driver_flightsql.dll",
        )),
        _ => None,
    }
}

fn fetch_release_metadata(version: &str) -> Result<PyPiRelease, Box<dyn std::error::Error>> {
    let url = format!("{PYPI_BASE}/{PACKAGE_NAME}/{version}/json");
    let client = Client::new();
    let response = client.get(url).send()?;
    if !response.status().is_success() {
        return Err(io::Error::other(format!(
            "Failed to fetch PyPI metadata: HTTP {}",
            response.status()
        ))
        .into());
    }
    Ok(response.json()?)
}

fn download_wheel(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut response = client.get(url).send()?;
    if !response.status().is_success() {
        return Err(io::Error::other(format!(
            "Failed to download FlightSQL wheel: HTTP {}",
            response.status()
        ))
        .into());
    }

    let mut bytes = Vec::new();
    response.copy_to(&mut bytes)?;
    Ok(bytes)
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let actual = format!("{digest:x}");
    if actual != expected {
        return Err(io::Error::other("Wheel checksum mismatch").into());
    }
    Ok(())
}

fn extract_library_from_wheel(
    wheel: &[u8],
    lib_filename: &str,
    dest: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cursor = std::io::Cursor::new(wheel);
    let mut archive = ZipArchive::new(cursor)?;
    let mut lib_entry = None;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name().to_owned();
        if name.ends_with(lib_filename) {
            lib_entry = Some((i, name));
            break;
        }
    }

    let (index, source_name) = lib_entry.ok_or_else(|| {
        io::Error::other(format!(
            "Wheel did not contain {lib_filename}; searched {} entries",
            archive.len()
        ))
    })?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut entry = archive.by_index(index)?;
    let mut output = File::create(dest)?;
    io::copy(&mut entry, &mut output)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(dest, perms)?;
    }

    println!(
        "cargo:warning=Copied ADBC FlightSQL driver from {source_name} into {}",
        dest.display()
    );

    Ok(())
}

#[derive(Debug, Deserialize)]
struct PyPiRelease {
    urls: Vec<PyPiFile>,
}

#[derive(Debug, Deserialize)]
struct PyPiFile {
    filename: String,
    url: String,
    #[serde(default)]
    digests: PyPiDigests,
}

#[derive(Debug, Default, Deserialize)]
struct PyPiDigests {
    #[serde(default)]
    sha256: Option<String>,
}

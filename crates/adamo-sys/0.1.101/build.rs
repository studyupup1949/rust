// build.rs — locate the precompiled libadamo for the host target and emit
// the link flags the Rust compiler needs.
//
// Resolution order:
//   1. $ADAMO_LIB_DIR (escape hatch for local dev and offline builds).
//   2. prebuilt/$TARGET/ inside this crate (the shipped artifact —
//      aarch64-unknown-linux-gnu / Jetson, so robot builds stay offline).
//   3. A version- and sha256-pinned download from install.adamohq.com for
//      the desktop targets listed in prebuilt/remote.txt. The artifact is
//      the same libadamo that ships in the C SDK tarball for that release.
//      Override the base URL with $ADAMO_SYS_REMOTE_BASE (any scheme curl
//      accepts, including file://).

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

/// Targets whose libadamo ships inside the crate tarball (offline builds).
const IN_CRATE_TARGETS: &[&str] = &["aarch64-unknown-linux-gnu"];

const DEFAULT_REMOTE_BASE: &str = "https://install.adamohq.com/sdk";

fn main() {
    println!("cargo:rerun-if-env-changed=ADAMO_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ADAMO_SYS_REMOTE_BASE");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=prebuilt/remote.txt");

    // docs.rs can't link against a real libadamo. Short-circuit so doc
    // builds succeed without the binaries present.
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let target = env::var("TARGET").expect("cargo sets TARGET");
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let remote = parse_remote(&crate_dir.join("prebuilt").join("remote.txt"));

    let lib = if let Ok(custom) = env::var("ADAMO_LIB_DIR") {
        let dir = PathBuf::from(custom);
        println!("cargo:rerun-if-changed={}", dir.display());
        pick_library(&dir).unwrap_or_else(|| {
            panic!(
                "adamo-sys: ADAMO_LIB_DIR is set but no libadamo.so/.dylib (or .xz) found in {}",
                dir.display()
            )
        })
    } else if IN_CRATE_TARGETS.contains(&target.as_str()) {
        let dir = crate_dir.join("prebuilt").join(&target);
        println!("cargo:rerun-if-changed={}", dir.display());
        pick_library(&dir).unwrap_or_else(|| {
            panic!(
                "adamo-sys: no libadamo found in {}. Set ADAMO_LIB_DIR to override.",
                dir.display()
            )
        })
    } else if let Some(entry) = remote.get(target.as_str()) {
        fetch_remote(&target, entry, &out_dir)
    } else {
        let mut supported: Vec<&str> = IN_CRATE_TARGETS.to_vec();
        supported.extend(remote.keys().map(String::as_str));
        supported.sort_unstable();
        supported.dedup();
        panic!(
            "adamo-sys: target `{target}` is not currently supported. \
             Supported targets: {}. \
             Set ADAMO_LIB_DIR to override if you have a libadamo built for your host.",
            supported.join(", ")
        );
    };

    // Stage the library into OUT_DIR so the link-search path is stable
    // regardless of where the source libadamo actually lives.
    let staged = out_dir.join(lib.staged_name);
    stage_library(&lib, &staged).unwrap_or_else(|err| {
        panic!(
            "adamo-sys: failed to stage {} to {}: {err}",
            lib.path.display(),
            staged.display()
        )
    });

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=adamo");
    // Absolute rpath so `cargo run` / `cargo test` work without
    // LD_LIBRARY_PATH. `rustc-link-arg` applies to any bin/example/test
    // that ends up depending on this crate. Downstream distributable
    // binaries should override with $ORIGIN (ship libadamo.so alongside
    // the executable).
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", out_dir.display());

    // Surface the staged library path to downstream crates via
    // DEP_ADAMO_LIB_DIR.
    println!("cargo:lib_dir={}", out_dir.display());
}

struct RemoteEntry {
    version: String,
    sha256: String,
}

/// prebuilt/remote.txt: one entry per downloadable target, whitespace
/// separated — `<target-triple> <artifact-version> <sha256>`. `#` starts a
/// comment. Maintained by scripts/pin-remote.sh at release time.
fn parse_remote(path: &Path) -> BTreeMap<String, RemoteEntry> {
    let mut map = BTreeMap::new();
    let Ok(text) = fs::read_to_string(path) else {
        return map;
    };
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split_whitespace();
        let (Some(target), Some(version), Some(sha256)) =
            (fields.next(), fields.next(), fields.next())
        else {
            panic!("adamo-sys: malformed line in {}: {line:?}", path.display());
        };
        map.insert(
            target.to_string(),
            RemoteEntry {
                version: version.to_string(),
                sha256: sha256.to_ascii_lowercase(),
            },
        );
    }
    map
}

fn dylib_ext(target: &str) -> &'static str {
    if target.contains("apple") { "dylib" } else { "so" }
}

/// Download the pinned artifact for `target` into OUT_DIR (reusing a prior
/// download when its checksum still matches) and verify it.
fn fetch_remote(target: &str, entry: &RemoteEntry, out_dir: &Path) -> LibrarySource {
    let ext = dylib_ext(target);
    let file_name = format!("libadamo-{target}.{ext}.xz");
    let dest = out_dir.join(&file_name);

    if !(dest.is_file() && sha256_hex(&dest) == entry.sha256) {
        let base =
            env::var("ADAMO_SYS_REMOTE_BASE").unwrap_or_else(|_| DEFAULT_REMOTE_BASE.to_string());
        let url = format!("{base}/v{}/{file_name}", entry.version);
        let tmp = out_dir.join(format!("{file_name}.part"));
        let status = Command::new("curl")
            .args(["--fail", "--location", "--silent", "--show-error", "--retry", "3"])
            .arg("-o")
            .arg(&tmp)
            .arg(&url)
            .status()
            .unwrap_or_else(|err| {
                panic!(
                    "adamo-sys: failed to run curl ({err}). curl is required to fetch the \
                     precompiled libadamo for `{target}`; for offline builds set ADAMO_LIB_DIR."
                )
            });
        if !status.success() {
            panic!(
                "adamo-sys: download of {url} failed ({status}). \
                 For offline builds set ADAMO_LIB_DIR to a directory containing libadamo."
            );
        }
        let actual = sha256_hex(&tmp);
        if actual != entry.sha256 {
            let _ = fs::remove_file(&tmp);
            panic!(
                "adamo-sys: checksum mismatch for {url}: expected {}, got {actual}",
                entry.sha256
            );
        }
        fs::rename(&tmp, &dest)
            .unwrap_or_else(|err| panic!("adamo-sys: failed to move download into place: {err}"));
    }

    LibrarySource {
        path: dest,
        staged_name: if ext == "dylib" { "libadamo.dylib" } else { "libadamo.so" },
        compressed_xz: true,
    }
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path)
        .unwrap_or_else(|err| panic!("adamo-sys: failed to read {}: {err}", path.display()));
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

struct LibrarySource {
    path: PathBuf,
    staged_name: &'static str,
    compressed_xz: bool,
}

fn pick_library(dir: &Path) -> Option<LibrarySource> {
    for name in ["libadamo.so", "libadamo.dylib"] {
        let p = dir.join(name);
        if p.is_file() {
            return Some(LibrarySource {
                path: p,
                staged_name: name,
                compressed_xz: false,
            });
        }
    }
    for (compressed, staged) in [
        ("libadamo.so.xz", "libadamo.so"),
        ("libadamo.dylib.xz", "libadamo.dylib"),
    ] {
        let p = dir.join(compressed);
        if p.is_file() {
            return Some(LibrarySource {
                path: p,
                staged_name: staged,
                compressed_xz: true,
            });
        }
    }
    None
}

fn stage_library(lib: &LibrarySource, staged: &Path) -> Result<(), String> {
    if !lib.compressed_xz {
        fs::copy(&lib.path, staged)
            .map(|_| ())
            .map_err(|err| err.to_string())
    } else {
        let input = File::open(&lib.path).map_err(|err| err.to_string())?;
        let output = File::create(staged).map_err(|err| err.to_string())?;
        let mut input = BufReader::new(input);
        let mut output = BufWriter::new(output);
        lzma_rs::xz_decompress(&mut input, &mut output).map_err(|err| err.to_string())
    }
}

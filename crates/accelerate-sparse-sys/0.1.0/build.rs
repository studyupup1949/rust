//! Builds the C shim and emits the link directives for Accelerate.
//!
//! Accelerate is an Apple framework, so every target other than macOS gets no native build at
//! all and the crate compiles to an empty library. That path exists so this crate can sit in a
//! cross-platform dependency tree without breaking it.
//!
//! There is deliberately no SDK-path discovery and no `bindgen`: the only header compiled here
//! is the shim's own, and the Rust declarations for it are written by hand.
//!
//! Two capabilities are probed against the active SDK, each enabling a define the shim compiles
//! against and a `cfg` the tests use: `ACCSP_HAVE_LU` for Accelerate's LU factorization kinds
//! (macOS 15.5+), and `ACCSP_HAVE_INERTIA` for `SparseGetInertia` (macOS 13.0+). Neither changes
//! the shape of the public API — the LU constants carry their fixed Apple values either way, and
//! the inertia entry points exist either way, reporting `ACCSP_STATUS_UNSUPPORTED_OS` when the
//! SDK did not provide the function. What each probe gates inside the shim differs: LU's turns on
//! its `_Static_assert` drift checks and the runtime OS guard around them, while inertia's has no
//! constants to check and gates the call site itself, which is replaced by a stub reporting that
//! it is unavailable.

use std::env;
use std::process::Stdio;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(accsp_have_lu)");
    println!("cargo::rustc-check-cfg=cfg(accsp_have_inertia)");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/shim.c");
    println!("cargo:rerun-if-changed=src/shim.h");
    println!("cargo:rerun-if-env-changed=ACCSP_DISABLE_LU");
    println!("cargo:rerun-if-env-changed=ACCSP_DISABLE_INERTIA");
    // The DOCS_RS branch below skips the native build, so its output depends on this variable.
    // Without tracking it, toggling DOCS_RS reuses a cached skip and links fail on missing symbols.
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    // Which SDK is active decides both probes below, and their answers are cached in the build
    // fingerprint. Without this, switching Xcode with `xcode-select` changes the available SDK
    // while leaving every declared input untouched, so a capability probed as absent stays absent
    // until something else forces a rebuild.
    println!("cargo:rerun-if-env-changed=SDKROOT");
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");

    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "macos" {
        return;
    }

    // docs.rs cross-compiles from a Linux host with the macOS target selected, so
    // CARGO_CFG_TARGET_OS reports "macos" while no Apple SDK is present. Documenting needs no
    // native library, and compiling the shim against a missing SDK would fail the build before
    // rustdoc runs. docs.rs sets DOCS_RS for exactly this case.
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let mut build = cc::Build::new();
    build
        .file("src/shim.c")
        .std("c11")
        .warnings(true)
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra");

    // Probed before any `define`, because `cc` passes accumulated definitions to every compiler
    // invocation it builds: probing afterwards would compile each probe with the defines the
    // earlier ones turned on, making the answers depend on the order of the calls.
    let have_lu = lu_supported(&build);
    let have_inertia = inertia_supported(&build);

    if have_lu {
        println!("cargo:rustc-cfg=accsp_have_lu");
        build.define("ACCSP_HAVE_LU", None);
    }
    if have_inertia {
        println!("cargo:rustc-cfg=accsp_have_inertia");
        build.define("ACCSP_HAVE_INERTIA", None);
    }

    build.compile("accelerate_sparse_shim");
    println!("cargo:rustc-link-lib=framework=Accelerate");
}

/// Whether the active SDK defines the LU factorization kinds (macOS 15.5+).
///
/// Detected by trying to compile a reference to `SparseFactorizationLU` rather than by parsing an
/// SDK version, so it tracks the symbol itself.
///
/// `ACCSP_DISABLE_LU` forces the result off. It suppresses this crate's LU support but not the
/// SDK: the shim is still compiled against the current SDK, whose inlined `SparseFactor` keeps
/// Accelerate's own LU trap. So it exercises the gated-out *build* on a host that could otherwise
/// build LU — it does not reproduce an older SDK, which would inline no LU path at all, and a
/// binary built this way must not be run on macOS < 15.5, where that trap would fire.
fn lu_supported(build: &cc::Build) -> bool {
    if env::var_os("ACCSP_DISABLE_LU").is_some() {
        return false;
    }
    compiles(
        build,
        "accsp_probe_lu.c",
        "int accsp_probe(void) { return (int)SparseFactorizationLU; }\n",
    )
}

/// Whether the active SDK declares `SparseGetInertia` (macOS 13.0+).
///
/// The probe is a call rather than a reference to the symbol: `SparseGetInertia` is one of
/// Accelerate's `__attribute__((overloadable))` functions, so naming it without arguments does not
/// select an overload and would not compile on any SDK. The call sits inside
/// `__builtin_available`, matching the shim, so that a deployment target below 13.0 does not make
/// the probe answer a question about availability warnings instead of about the SDK.
///
/// `ACCSP_DISABLE_INERTIA` forces the result off. Unlike the LU switch this carries no trap
/// hazard — `SparseGetInertia` is an ordinary exported function, not an inlined dispatch — so a
/// binary built with it suppressed reports `ACCSP_STATUS_UNSUPPORTED_OS` and is safe to run
/// anywhere. It exists to exercise the gated-out build.
fn inertia_supported(build: &cc::Build) -> bool {
    if env::var_os("ACCSP_DISABLE_INERTIA").is_some() {
        return false;
    }
    compiles(
        build,
        "accsp_probe_inertia.c",
        "int accsp_probe(SparseOpaqueFactorization_Double f) { \
         int positive, zero, negative; \
         if (__builtin_available(macOS 13.0, *)) { \
         return SparseGetInertia(f, &positive, &zero, &negative); } \
         return 0; }\n",
    )
}

/// Whether `body`, compiled against `<Accelerate/Accelerate.h>`, passes a syntax-only build.
///
/// Every probe error reports `false`, including a missing `OUT_DIR`, an unwritable probe file, or a
/// compiler that will not start. A false negative disables the capability; a false positive causes
/// a later link failure.
///
/// Clang before version 16 only warns about an undeclared function, so the probe makes that
/// diagnostic an error. The flag is added after `get_compiler` so it follows any caller-supplied
/// `CFLAGS` that would restore the warning.
fn compiles(build: &cc::Build, file_name: &str, body: &str) -> bool {
    let Ok(out_dir) = env::var("OUT_DIR") else {
        return false;
    };
    let probe = std::path::Path::new(&out_dir).join(file_name);
    let source = format!("#include <Accelerate/Accelerate.h>\n{body}");
    if std::fs::write(&probe, source).is_err() {
        return false;
    }

    let mut command = build.get_compiler().to_command();
    command
        .arg("-Werror=implicit-function-declaration")
        .arg("-fsyntax-only")
        .arg(&probe)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    matches!(command.status(), Ok(status) if status.success())
}

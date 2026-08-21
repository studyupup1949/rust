//! DRM module — Phase 2: Widevine + forensic watermarking.
//! Enable with feature flag: `--features drm`

/// Widevine license acquisition (Phase 2).
#[cfg(feature = "drm")]
pub mod widevine;

/// Placeholder to satisfy mod declaration in lib.rs.
#[cfg(not(feature = "drm"))]
pub fn drm_enabled() -> bool {
    false
}

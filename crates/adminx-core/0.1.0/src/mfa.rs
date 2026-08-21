// adminx-core/src/mfa.rs
//
// TOTP-based multi-factor auth primitives: secret generation, QR provisioning,
// code verification, and one-time backup codes. Framework- and storage-neutral;
// the auth handlers and adapters build the flow on top of these helpers.

use crate::error::CoreError;
use totp_rs::{Algorithm, Secret, TOTP};

/// Issuer shown in the authenticator app.
const ISSUER: &str = "adminx";
/// Number of one-time backup codes generated at enable time.
pub const BACKUP_CODE_COUNT: usize = 10;

/// Generate a fresh base32 TOTP secret to persist on the user row.
pub fn generate_secret() -> String {
    Secret::generate_secret().to_encoded().to_string()
}

/// Build a `TOTP` from a stored base32 secret and the user's account label.
fn totp_for(secret_b32: &str, account: &str) -> Result<TOTP, CoreError> {
    let bytes = Secret::Encoded(secret_b32.to_string())
        .to_bytes()
        .map_err(|e| CoreError::Internal(format!("invalid mfa secret: {e:?}")))?;
    TOTP::new(
        Algorithm::SHA1,
        6,
        1, // allow +/- 1 step of clock skew
        30,
        bytes,
        Some(ISSUER.to_string()),
        account.to_string(),
    )
    .map_err(|e| CoreError::Internal(format!("totp init: {e}")))
}

/// `otpauth://` URL to encode into the setup QR code.
pub fn provisioning_url(secret_b32: &str, account: &str) -> Result<String, CoreError> {
    Ok(totp_for(secret_b32, account)?.get_url())
}

/// True when `code` is the valid current TOTP for this secret/account.
pub fn check_code(secret_b32: &str, account: &str, code: &str) -> bool {
    match totp_for(secret_b32, account) {
        Ok(totp) => totp.check_current(code.trim()).unwrap_or(false),
        Err(_) => false,
    }
}

/// Render arbitrary data (an otpauth URL) as an inline SVG QR code. SVG keeps the
/// dependency light — no raster/image encoders.
pub fn qr_svg(data: &str) -> Result<String, CoreError> {
    use qrcode::render::svg;
    use qrcode::QrCode;
    let code =
        QrCode::new(data.as_bytes()).map_err(|e| CoreError::Internal(format!("qr encode: {e}")))?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(220, 220)
        .quiet_zone(true)
        .build())
}

// ===== Backup / recovery codes =====

/// Generate `BACKUP_CODE_COUNT` human-friendly one-time codes (shown once, in
/// plain text). Format: `1234-5678`.
pub fn generate_backup_codes() -> Vec<String> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..BACKUP_CODE_COUNT)
        .map(|_| {
            let n: u32 = rng.gen_range(0..100_000_000);
            let s = format!("{n:08}");
            format!("{}-{}", &s[..4], &s[4..])
        })
        .collect()
}

/// Hash backup codes for storage. Returns a JSON array of bcrypt hashes.
pub fn hash_backup_codes(codes: &[String]) -> Result<String, CoreError> {
    let hashes = codes
        .iter()
        .map(|c| crate::auth::hash_password(c))
        .collect::<Result<Vec<String>, _>>()?;
    serde_json::to_string(&hashes).map_err(|e| CoreError::Internal(e.to_string()))
}

/// If `code` matches one of the stored (JSON array) hashes, consume it and return
/// the remaining hashes as JSON. Returns `None` when nothing matched.
pub fn consume_backup_code(stored_json: &str, code: &str) -> Option<String> {
    let mut hashes: Vec<String> = serde_json::from_str(stored_json).ok()?;
    let code = code.trim();
    let pos = hashes
        .iter()
        .position(|h| crate::auth::verify_password(code, h))?;
    hashes.remove(pos);
    serde_json::to_string(&hashes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_roundtrips_to_a_working_totp() {
        let secret = generate_secret();
        let totp = totp_for(&secret, "user@example.com").unwrap();
        let code = totp.generate_current().unwrap();
        assert!(check_code(&secret, "user@example.com", &code));
        assert!(!check_code(&secret, "user@example.com", "000000"));
    }

    #[test]
    fn provisioning_url_is_otpauth() {
        let secret = generate_secret();
        let url = provisioning_url(&secret, "user@example.com").unwrap();
        assert!(url.starts_with("otpauth://totp/"));
    }

    #[test]
    fn backup_codes_are_one_time() {
        let codes = generate_backup_codes();
        assert_eq!(codes.len(), BACKUP_CODE_COUNT);
        let stored = hash_backup_codes(&codes).unwrap();

        // A wrong code consumes nothing.
        assert!(consume_backup_code(&stored, "0000-0000").is_none() || !codes.contains(&"0000-0000".to_string()));

        // A valid code is accepted once, then gone.
        let remaining = consume_backup_code(&stored, &codes[0]).expect("first use accepted");
        assert!(consume_backup_code(&remaining, &codes[0]).is_none());
    }
}

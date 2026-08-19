use std::collections::BTreeMap;

use crate::crc::{crc16_hex_lower, crc16_hex_upper, is_hex4};
use crate::error::Hj212Error;
use crate::packet::Hj212Packet;

/// Parse one HJ212 frame like:
/// `##0453QN=...;ST=..;...;CP=&&DataTime=...;a21005-Rtd=1.1,...;&&c0c1`
///
/// Notes:
/// - Accepts 3-4 digit length fields.
/// - CRC validation is performed if CRC exists; otherwise it's skipped (truncated frames).
pub fn parse_frame(input: &str) -> Result<Hj212Packet, Hj212Error> {
    let s = input.trim();
    if !s.starts_with("##") {
        return Err(Hj212Error::MissingPrefix);
    }

    // Extract length digits immediately after ## (3-4 digits)
    let mut idx = 2;
    let bytes = s.as_bytes();
    let mut len_digits = String::new();
    while idx < bytes.len() && len_digits.len() < 4 {
        let c = bytes[idx] as char;
        if c.is_ascii_digit() {
            len_digits.push(c);
            idx += 1;
        } else {
            break;
        }
    }
    if len_digits.len() < 3 {
        return Err(Hj212Error::InvalidLength);
    }
    let payload_len = len_digits.parse::<usize>().map_err(|_| Hj212Error::InvalidLength)?;
    let length_hint = Some(payload_len);

    let payload_start = idx;
    let payload_end = payload_start + payload_len;
    let crc_end = payload_end + 4;

    if s.as_bytes().len() >= crc_end {
        let payload = String::from_utf8_lossy(&s.as_bytes()[payload_start..payload_end]).to_string();
        let crc_candidate = String::from_utf8_lossy(&s.as_bytes()[payload_end..crc_end]).to_string();
        let crc_hex = if is_hex4(crc_candidate.trim()) {
            Some(crc_candidate.trim().to_lowercase())
        } else {
            return Err(Hj212Error::InvalidCrc);
        };

        let expected = crc16_hex_lower(payload.as_bytes());
        if expected != *crc_hex.as_ref().unwrap() {
            return Err(Hj212Error::CrcMismatch {
                expected,
                got: crc_hex.unwrap(),
            });
        }

        return parse_payload(length_hint, payload, Some(expected));
    }

    // Truncated: best-effort parse without CRC validation.
    let rest = &s[idx..];
    parse_payload(length_hint, rest.to_string(), None)
}

/// Parse a **standard** HJ212 frame strictly (HJ 212—2025 6.3.2):
/// `##` + 4-digit length + payload + 4-hex CRC + `\r\n`.
///
/// Differences vs [`parse_frame`]:
/// - Requires exactly 4 length digits
/// - Requires CRC and validates it
/// - Requires trailing `\r\n`
pub fn parse_frame_strict(input: &str) -> Result<Hj212Packet, Hj212Error> {
    if !input.starts_with("##") {
        return Err(Hj212Error::MissingPrefix);
    }
    if !input.ends_with("\r\n") {
        return Err(Hj212Error::MissingSuffix);
    }

    let bytes = input.as_bytes();
    if bytes.len() < 2 + 4 + 4 + 2 {
        return Err(Hj212Error::InvalidLength);
    }

    let len_str = std::str::from_utf8(&bytes[2..6]).map_err(|_| Hj212Error::InvalidLength)?;
    if !len_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(Hj212Error::InvalidLength);
    }
    let payload_len = len_str.parse::<usize>().map_err(|_| Hj212Error::InvalidLength)?;

    // Layout: ## + 4(len) + payload(payload_len) + CRC(4) + CRLF(2)
    let payload_start = 6;
    let payload_end = payload_start + payload_len;
    let crc_end = payload_end + 4;
    let suffix_end = crc_end + 2;

    if bytes.len() != suffix_end {
        return Err(Hj212Error::InvalidLength);
    }

    let payload = String::from_utf8_lossy(&bytes[payload_start..payload_end]).to_string();
    let crc_candidate = String::from_utf8_lossy(&bytes[payload_end..crc_end]).to_string();
    if !is_hex4(crc_candidate.trim()) {
        return Err(Hj212Error::InvalidCrc);
    }
    let got = crc_candidate.trim().to_lowercase();
    let expected = crc16_hex_lower(payload.as_bytes());
    if expected != got {
        return Err(Hj212Error::CrcMismatch { expected, got });
    }

    parse_payload(Some(payload_len), payload, Some(expected))
}

fn parse_payload(length_hint: Option<usize>, payload: String, crc_hex: Option<String>) -> Result<Hj212Packet, Hj212Error> {
    let (head, cp_body) = payload.split_once("CP=&&").ok_or(Hj212Error::MissingCp)?;

    let mut qn = None;
    let mut st = None;
    let mut cn = None;
    let mut pw = None;
    let mut mn = None;
    let mut flag = None;

    for part in head.split(';').filter(|p| !p.trim().is_empty()) {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            match k {
                "QN" => qn = Some(v.to_string()),
                "ST" => st = Some(v.to_string()),
                "CN" => cn = Some(v.to_string()),
                "PW" => pw = Some(v.to_string()),
                "MN" => mn = Some(v.to_string()),
                "Flag" => flag = Some(v.to_string()),
                _ => {}
            }
        }
    }

    let cp_body = cp_body.strip_suffix("&&").unwrap_or(cp_body).trim();
    let mut cp: BTreeMap<String, String> = BTreeMap::new();
    for part in cp_body.split(';').filter(|p| !p.trim().is_empty()) {
        if let Some((k, v)) = part.split_once('=') {
            cp.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    let data_time = cp.get("DataTime").cloned();

    Ok(Hj212Packet {
        length_hint,
        payload,
        crc_hex,
        qn,
        st,
        cn,
        pw,
        mn,
        flag,
        cp,
        data_time,
    })
}

/// Wrap a payload into a **standard** HJ212 frame (HJ 212—2025 6.3.2):
/// `##` + 4-digit length + payload + 4-hex CRC + `\r\n`.
///
/// Notes:
/// - Length is the ASCII byte length of `payload`
/// - CRC is HJ 212—2025 "ANSI CRC16" over the payload, rendered as **uppercase** hex
pub fn build_frame(payload: &str) -> String {
    build_frame_standard(payload)
}

/// Build a **standard** HJ212 frame (HJ 212—2025 6.3.2):
/// `##` + 4-digit length + payload + 4-hex CRC + `\r\n`.
///
/// Notes:
/// - Length is the ASCII byte length of `payload`
/// - CRC is HJ 212—2025 "ANSI CRC16" over the payload, rendered as **uppercase** hex
pub fn build_frame_standard(payload: &str) -> String {
    let len = payload.as_bytes().len();
    let crc = crc16_hex_upper(payload.as_bytes());
    format!("##{:04}{}{}\r\n", len, payload, crc)
}

/// Build a **compat** frame without the standard CRLF suffix.
///
/// This matches some historical/platform variants: `##{LEN}{PAYLOAD}{CRC}` with lowercase CRC and no `\r\n`.
pub fn build_frame_compat(payload: &str) -> String {
    let len = payload.as_bytes().len();
    let crc = crc16_hex_lower(payload.as_bytes());
    format!("##{:04}{}{}", len, payload, crc)
}

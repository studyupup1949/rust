//! Minimal APK inspection. Replacement for the `apkutils` dependency used by
//! `install.py` — just enough to recover the `package` name from a binary
//! `AndroidManifest.xml`, which is all the install flow needs (launch uses
//! `monkey`, so the main activity is not required).

use std::io::Read;
use std::path::Path;

use crate::errors::{AdbError, Result};

/// Basic APK metadata.
#[derive(Debug, Clone)]
pub struct ApkInfo {
    pub package_name: String,
}

/// Read `AndroidManifest.xml` from `apk_path` and extract the package name.
pub fn parse_apk(apk_path: &Path) -> Result<ApkInfo> {
    let file = std::fs::File::open(apk_path)
        .map_err(|e| AdbError::adb(format!("cannot open apk {}: {e}", apk_path.display())))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| AdbError::adb(format!("not a valid apk/zip: {e}")))?;
    let mut manifest = zip
        .by_name("AndroidManifest.xml")
        .map_err(|_| AdbError::adb("AndroidManifest.xml not found in apk"))?;
    let mut buf = Vec::new();
    manifest
        .read_to_end(&mut buf)
        .map_err(|e| AdbError::adb(format!("cannot read manifest: {e}")))?;
    let package_name = axml_package_name(&buf)
        .ok_or_else(|| AdbError::adb("cannot find package name in manifest"))?;
    Ok(ApkInfo { package_name })
}

// AXML chunk types.
const RES_STRING_POOL_TYPE: u16 = 0x0001;
const RES_XML_START_ELEMENT_TYPE: u16 = 0x0102;
const UTF8_FLAG: u32 = 1 << 8;

fn u16le(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// Walk the binary XML, extracting the `package` attribute of the `manifest`
/// start element. Returns `None` on any structural surprise.
fn axml_package_name(data: &[u8]) -> Option<String> {
    if data.len() < 8 {
        return None;
    }
    // File header: type(u16) headerSize(u16) size(u32). Chunks follow.
    let mut pos = 8usize;
    let mut strings: Vec<String> = Vec::new();

    while pos + 8 <= data.len() {
        let chunk_type = u16le(data, pos);
        let chunk_size = u32le(data, pos + 4) as usize;
        if chunk_size == 0 || pos + chunk_size > data.len() {
            break;
        }
        match chunk_type {
            RES_STRING_POOL_TYPE => {
                strings = parse_string_pool(&data[pos..pos + chunk_size]).unwrap_or_default();
            }
            RES_XML_START_ELEMENT_TYPE => {
                if let Some(pkg) = start_element_package(&data[pos..pos + chunk_size], &strings) {
                    return Some(pkg);
                }
            }
            _ => {}
        }
        pos += chunk_size;
    }
    None
}

fn parse_string_pool(chunk: &[u8]) -> Option<Vec<String>> {
    // Header: type,headerSize,size,stringCount,styleCount,flags,stringsStart,stylesStart
    if chunk.len() < 28 {
        return None;
    }
    let string_count = u32le(chunk, 8) as usize;
    let flags = u32le(chunk, 16);
    let strings_start = u32le(chunk, 20) as usize;
    let is_utf8 = flags & UTF8_FLAG != 0;

    let mut out = Vec::with_capacity(string_count);
    let offsets_start = 28;
    for i in 0..string_count {
        let off_pos = offsets_start + i * 4;
        if off_pos + 4 > chunk.len() {
            break;
        }
        let str_off = strings_start + u32le(chunk, off_pos) as usize;
        if str_off >= chunk.len() {
            out.push(String::new());
            continue;
        }
        out.push(read_pool_string(chunk, str_off, is_utf8).unwrap_or_default());
    }
    Some(out)
}

fn read_pool_string(chunk: &[u8], off: usize, is_utf8: bool) -> Option<String> {
    if is_utf8 {
        // char length (u8/u16), then byte length (u8/u16), then bytes.
        let (_, o1) = read_len8(chunk, off)?;
        let (byte_len, o2) = read_len8(chunk, o1)?;
        let end = o2 + byte_len;
        if end > chunk.len() {
            return None;
        }
        Some(String::from_utf8_lossy(&chunk[o2..end]).into_owned())
    } else {
        // char length (u16 units), then UTF-16LE chars.
        let (char_len, o1) = read_len16(chunk, off)?;
        let end = o1 + char_len * 2;
        if end > chunk.len() {
            return None;
        }
        let units: Vec<u16> = chunk[o1..end]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        Some(String::from_utf16_lossy(&units))
    }
}

/// UTF-8 pool length: one or two bytes; high bit of first byte extends it.
fn read_len8(b: &[u8], o: usize) -> Option<(usize, usize)> {
    let first = *b.get(o)? as usize;
    if first & 0x80 != 0 {
        let second = *b.get(o + 1)? as usize;
        Some((((first & 0x7f) << 8) | second, o + 2))
    } else {
        Some((first, o + 1))
    }
}

/// UTF-16 pool length: one or two u16s; high bit of first unit extends it.
fn read_len16(b: &[u8], o: usize) -> Option<(usize, usize)> {
    let first = u16le(b, o) as usize;
    if first & 0x8000 != 0 {
        let second = u16le(b, o + 2) as usize;
        Some((((first & 0x7fff) << 16) | second, o + 4))
    } else {
        Some((first, o + 2))
    }
}

/// Parse a START_ELEMENT chunk; if it names `manifest`, return its `package`.
fn start_element_package(chunk: &[u8], strings: &[String]) -> Option<String> {
    // chunk header(8) lineNumber(4) comment(4) ns(4) name(4) attrStart(2)
    // attrSize(2) attrCount(2) idIndex(2) classIndex(2) styleIndex(2)
    if chunk.len() < 36 {
        return None;
    }
    let name_idx = u32le(chunk, 20) as usize;
    let name = strings.get(name_idx)?;
    if name != "manifest" {
        return None;
    }
    let attr_start = u16le(chunk, 24) as usize;
    let attr_count = u16le(chunk, 28) as usize;
    // Attributes begin at chunk offset 8 + attr_start; each is 20 bytes:
    // ns(4) name(4) rawValue(4) size(2) res0(1) dataType(1) data(4)
    let base = 8 + attr_start;
    for i in 0..attr_count {
        let a = base + i * 20;
        if a + 20 > chunk.len() {
            break;
        }
        let attr_name_idx = u32le(chunk, a + 4) as usize;
        if strings.get(attr_name_idx).map(String::as_str) != Some("package") {
            continue;
        }
        let raw_value = u32le(chunk, a + 8);
        if raw_value != 0xffff_ffff {
            if let Some(s) = strings.get(raw_value as usize) {
                return Some(s.clone());
            }
        }
        // Fall back to the typed string data index.
        let data = u32le(chunk, a + 16) as usize;
        if let Some(s) = strings.get(data) {
            return Some(s.clone());
        }
    }
    None
}

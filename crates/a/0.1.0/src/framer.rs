/// Incremental frame extractor for the ASCII HJ212 transport:
/// `##` + 4 digits length + payload + 4 hex CRC + optional `\r\n`.
#[derive(Debug, Default, Clone)]
pub struct Framer {
    buffer: Vec<u8>,
}

impl Framer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn next_frame(&mut self) -> Option<Vec<u8>> {
        try_extract_frame(&mut self.buffer)
    }
}

fn try_extract_frame(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Find start marker
    let start = buffer
        .windows(2)
        .position(|w| w == b"##")
        .unwrap_or_else(|| {
            buffer.clear();
            usize::MAX
        });
    if start == usize::MAX {
        return None;
    }
    if start > 0 {
        buffer.drain(0..start);
    }

    if buffer.len() < 2 + 3 {
        return None;
    }

    // Parse length digits after ## (3-4 digits)
    let mut len_digits = Vec::new();
    let mut idx = 2;
    while idx < buffer.len() && len_digits.len() < 4 {
        let c = buffer[idx] as char;
        if c.is_ascii_digit() {
            len_digits.push(buffer[idx]);
            idx += 1;
        } else {
            break;
        }
    }
    if len_digits.len() < 3 {
        return None;
    }
    let len_str = String::from_utf8_lossy(&len_digits);
    let payload_len = len_str.parse::<usize>().ok()?;
    let digits_count = len_digits.len();

    // Expect payload + 4 hex CRC (+ optional CRLF).
    let total_needed = 2 + digits_count + payload_len + 4;
    if buffer.len() < total_needed {
        return None;
    }

    // Prefer consuming CRLF suffix if present.
    if buffer.len() >= total_needed + 2 && &buffer[total_needed..total_needed + 2] == b"\r\n" {
        return Some(buffer.drain(0..total_needed + 2).collect());
    }

    Some(buffer.drain(0..total_needed).collect())
}

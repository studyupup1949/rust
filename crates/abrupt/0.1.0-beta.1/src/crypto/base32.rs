const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const PADDING: char = '=';

pub fn encode(input: &[u8]) -> String {
    let mut encoded = String::new();
    let mut temp = 0u32;
    let mut bits = 0;

    for &byte in input {
        temp = (temp << 8) + byte as u32;
        bits += 8;

        while bits >= 5 {
            bits -= 5;
            let index = ((temp >> bits) & 0x1F) as usize;
            encoded.push(ALPHABET.chars().nth(index).unwrap());
        }
    }

    if bits > 0 {
        temp <<= 5 - bits;
        let index = (temp & 0x1F) as usize;
        encoded.push(ALPHABET.chars().nth(index).unwrap());
    }

    while encoded.len() % 8 != 0 {
        encoded.push(PADDING);
    }

    encoded
}

pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    let clean_input = input.trim_end_matches(PADDING);

    for c in clean_input.chars() {
        if let Some(pos) = ALPHABET.find(c) {
            buffer = (buffer << 5) | pos as u32;
            bits += 5;

            if bits >= 8 {
                bits -= 8;
                bytes.push((buffer >> bits) as u8);
            }
        } else {
            return Err(format!("Invalid character '{}' in input.", c));
        }
    }

    Ok(bytes)
}

const HEX_CHARS: &str = "0123456789ABCDEF";

pub fn encode(input: &[u8]) -> String {
    input
        .iter()
        .map(|&byte| {
            let high = (byte >> 4) as usize;
            let low = (byte & 0x0F) as usize;
            format!(
                "{}{}",
                HEX_CHARS.chars().nth(high).unwrap(),
                HEX_CHARS.chars().nth(low).unwrap()
            )
        })
        .collect()
}

pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut chars = input.chars();

    while let (Some(high_char), Some(low_char)) = (chars.next(), chars.next()) {
        let high = HEX_CHARS
            .find(high_char)
            .ok_or_else(|| format!("Invalid hex character '{}'", high_char))?;
        let low = HEX_CHARS
            .find(low_char)
            .ok_or_else(|| format!("Invalid hex character '{}'", low_char))?;
        bytes.push(((high << 4) | low) as u8);
    }

    if chars.next().is_some() {
        return Err("Hex input has an odd length".to_string());
    }

    Ok(bytes)
}

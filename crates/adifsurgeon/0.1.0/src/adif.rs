use regex::Regex;
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

pub fn get_header() -> Vec<String> {
    let mut result = Vec::new();

    result.push("ADIF Surgeon Export\n".to_string());
    result.push(format!(
        "<PROGRAMID:{}>{} \n",
        "ADIFSurgeon".len(),
        "ADIFSurgeon"
    ));

    result
}

pub fn write_header<W: Write>(header: &[String], output: &mut W) -> io::Result<()> {
    for line in header.iter() {
        output.write_all(line.as_bytes())?;
    }
    output.write_all("<eoh>\n".as_bytes())?;
    output.flush()?;

    Ok(())
}

pub fn write_record<W: Write>(record: &HashMap<String, String>, output: &mut W) -> io::Result<()> {
    for (k, v) in record.iter() {
        output.write_all(format!("<{}:{}>{} \n", k, v.len(), v).as_bytes())?;
    }
    output.write_all("<eor>\n".as_bytes())?;
    Ok(())
}

pub fn parse_record(input: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut position = 0;
    let bytes = input.as_bytes();

    while position < bytes.len() {
        // Find the opening angle bracket
        if let Some(start_tag) = input[position..].find('<') {
            position += start_tag;

            // Find the closing angle bracket and colon
            if let Some(end_tag) = input[position..].find(':') {
                let key = &input[position + 1..position + end_tag];
                position += end_tag + 1;

                // Parse the count of characters
                if let Some(end_count) = input[position..].find('>') {
                    let count_str = &input[position..position + end_count];
                    if let Ok(count) = count_str.parse::<usize>() {
                        position += end_count + 1;

                        // Extract the value based on count
                        if position + count <= input.len() {
                            let value = &input[position..position + count];
                            result.insert(key.to_string(), value.to_string());
                            position += count + 1; // +1 for the space after value
                        }
                    }
                }
            }
        } else {
            break;
        }
    }

    result
}

pub fn parse_header<R: BufRead>(reader: &mut R) -> io::Result<String> {
    let eoh_regex = Regex::new(r"(?i)<eoh>").unwrap();

    let mut header = String::new();

    for line in reader.lines() {
        let line = line?;

        if eoh_regex.is_match(&line) {
            // Split the line at <EOR> (case-insensitive)
            let parts: Vec<&str> = eoh_regex.split(&line).collect();

            // Add the part before <EOR> to current record
            header.push_str(parts[0]);
            break;
        } else {
            // Just add the line to the current record
            if !header.is_empty() {
                header.push('\n');
            }
            header.push_str(&line);
        }
    }

    Ok(header)
}

pub fn parse_records<R: BufRead>(reader: &mut R) -> io::Result<Vec<String>> {
    let eor_regex = Regex::new(r"(?i)<eor>").unwrap();

    let mut records = Vec::new();
    let mut current_record = String::new();

    for line in reader.lines() {
        let line = line?;

        if eor_regex.is_match(&line) {
            // Split the line at <EOR> (case-insensitive)
            let parts: Vec<&str> = eor_regex.split(&line).collect();

            // Add the part before <EOR> to current record
            current_record.push_str(parts[0]);

            // Save the current record if it's not empty
            if !current_record.is_empty() {
                records.push(current_record.trim().to_string());
                current_record = String::new();
            }

            // If there's content after <EOR>, start a new record with it
            if parts.len() > 1 {
                current_record = parts[1..].join("").to_string();
            }
        } else {
            // Just add the line to the current record
            if !current_record.is_empty() {
                current_record.push('\n');
            }
            current_record.push_str(&line);
        }
    }

    // Don't forget to add the last record if there's no final <EOR>
    if !current_record.is_empty() {
        records.push(current_record.trim().to_string());
    }

    Ok(records)
}

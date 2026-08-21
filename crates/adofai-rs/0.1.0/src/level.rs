use serde_jsonrc::Value;

fn replace_invalid(data: &mut String) {
    let mut chars = data.chars();

    if data.ends_with("\n") {
        chars.next_back();
    }

    if data.starts_with("﻿") {
        chars.next();
    }

    *data = chars.as_str().to_string();

    *data = data
        .replace(", ,", ", ")
        .replace("}\n", "},\n")
        .replace("},\n\t]", "}\n\t]")
        .replace(", },", " },")
        .replace(", }", " }")
        .replace("}\n", "},\n");
}

pub fn load(file_content: &str) -> Value {
    let mut f = file_content.clone().to_string();

    replace_invalid(&mut f);

    let data: Value = serde_jsonrc::from_str(&f.trim()).unwrap();

    data
}

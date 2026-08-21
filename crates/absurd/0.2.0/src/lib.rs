/// Looks for the `marker` within `text`, then adds `new_line` after it.
///
/// # Example
/// ```
/// use absurd::append_after_marker;
///
/// let text = r#"
///     pub mod bar;
///     pub mod foo;
/// "#.to_string();
/// let new_line = "    pub mod baz;".to_string();
/// let text = append_after_marker(text, "    pub mod", new_line);
/// assert_eq!(&text, r#"
///     pub mod bar;
///     pub mod baz;
///     pub mod foo;
/// "#);
/// ```
pub fn append_after_marker(text: String, marker: &str, new_line: String) -> String {
    let mut text = text;
    let mut mods = text
        .lines()
        .filter(|line| line.starts_with(marker))
        .map(|line| (text.find(line).unwrap_or(0), line))
        .filter(|(_, line)| line.starts_with(marker))
        .collect::<Vec<(usize, &str)>>();
    mods.push((0, &new_line));
    mods.sort_by(|a, b| a.1.cmp(&b.1));

    // Find the location of the search string.
    // Then find the element after it and extract it's index.
    // Then sub 1 to that index to add it before it.
    // Subtracting from the index means we don't have to add the length of
    // the string to the index.
    // Unwrap is safe because we pushed this string to the vec above.
    let pos = mods.iter().position(|m| m.1 == &new_line).unwrap();
    let pos = match mods.get(pos.saturating_add(1)) {
        // We can take the next element and insert in front of it.
        Some(pos) => pos.0,
        // Otherwise, we need to go back an element and find the end of that
        // line.
        None => {
            let pos = pos.saturating_sub(1);
            let mod_line = mods.get(pos).unwrap();
            mod_line.0 + mod_line.1.len() + 1
        },
    };

    // To prevent overflow, if the position is too large, then just push the
    // string.
    if pos > text.len() {
        text.push_str(format!("\n{}", new_line).as_str());
    } else {
        text.insert_str(pos, format!("{}\n", new_line).as_str());
    }

    text
}

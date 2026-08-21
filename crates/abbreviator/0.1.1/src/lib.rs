pub fn abbreviate(word: &str) -> String {
    let word_length = word.chars().count();
    if word_length < 3 {
        return word.to_string();
    }
    return get_first_letter(word)
        + (word_length - 2).to_string().as_str()
        + get_last_letter(word).as_str();
}

fn get_nth_letter(word: &str, index: u16) -> String {
    return word.chars().nth(index as usize).unwrap().to_string();
}

fn get_first_letter(word: &str) -> String {
    return get_nth_letter(word, 0);
}

fn get_last_letter(word: &str) -> String {
    return get_nth_letter(word, (word.chars().count() - 1) as u16);
}

#[cfg(test)]
mod tests {
    use crate::{abbreviate, get_first_letter, get_last_letter};

    #[test]
    fn test_get_first_letter() {
        assert_eq!(get_first_letter("hello"), "h");
        assert_eq!(get_first_letter("HELLO"), "H");
        assert_eq!(get_first_letter("a"), "a");
        // Todo:
        // Even though this function will never be called with an empty string
        // I have to fix this issue some time.
        // assert_eq!(get_first_letter(""), "");
    }

    #[test]
    fn test_get_last_letter() {
        assert_eq!(get_last_letter("hello"), "o");
        assert_eq!(get_last_letter("HELLO"), "O");
        assert_eq!(get_last_letter("a"), "a");
        // Todo:
        // Even though this function will never be called with an empty string
        // I have to fix this issue some time.
        // assert_eq!(get_last_letter(""), "");
    }

    #[test]
    fn test_abbreviate() {
        assert_eq!(abbreviate(""), "");
        assert_eq!(abbreviate("a"), "a");
        assert_eq!(abbreviate("ab"), "ab");
        assert_eq!(abbreviate("abc"), "a1c");
        assert_eq!(abbreviate("word"), "w2d");
        assert_eq!(abbreviate("localization"), "l10n");
        assert_eq!(abbreviate("internationalization"), "i18n");
        assert_eq!(abbreviate("pneumonoultramicroscopicsilicovolcanoconiosis"), "p43s");
    }
}

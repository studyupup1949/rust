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
    if word.is_empty() {
        return word.to_string();
    }

    return word.chars().nth(index as usize).unwrap().to_string();
}

fn get_first_letter(word: &str) -> String {
    return get_nth_letter(word, 0);
}

fn get_last_letter(word: &str) -> String {
    let index = word.chars().count().saturating_sub(1);
    return get_nth_letter(word, index as u16);
}

#[cfg(test)]
mod tests {
    use crate::{abbreviate, get_first_letter, get_last_letter};

    #[test]
    fn test_get_first_letter() {
        assert_eq!(get_first_letter("hello"), "h");
        assert_eq!(get_first_letter("HELLO"), "H");
        assert_eq!(get_first_letter("a"), "a");
        assert_eq!(get_first_letter(""), "");
    }

    #[test]
    fn test_get_last_letter() {
        assert_eq!(get_last_letter("hello"), "o");
        assert_eq!(get_last_letter("HELLO"), "O");
        assert_eq!(get_last_letter("a"), "a");
        assert_eq!(get_last_letter(""), "");
    }

    #[test]
    fn test_abbreviate() {
        // English
        assert_eq!(abbreviate(""), "");
        assert_eq!(abbreviate("a"), "a");
        assert_eq!(abbreviate("ab"), "ab");
        assert_eq!(abbreviate("abc"), "a1c");
        assert_eq!(abbreviate("word"), "w2d");
        assert_eq!(abbreviate("localization"), "l10n");
        assert_eq!(abbreviate("internationalization"), "i18n");
        assert_eq!(abbreviate("pneumonoultramicroscopicsilicovolcanoconiosis"), "p43s");

        // Greek
        assert_eq!(abbreviate("τι κάνεις"), "τ7ς");
        assert_eq!(abbreviate("Καλημέρα αρχηγέ"), "Κ13έ");
        assert_eq!(abbreviate("Άντε και στα δικά σου!"), "Ά20!");
        assert_eq!(abbreviate("σκουλικομερμυγκότρυπα"), "σ19α");
        assert_eq!(abbreviate("Άσπρη πέτρα ξέξασπρη κι απ' τον ήλιο ξεξασπρώτερη"), "Ά47η");
    }
}

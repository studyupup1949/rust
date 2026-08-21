use std::io;
use meval;
use rand::Rng;

pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn text_analyzer(input: &str) -> String {
    let word_count = input.split_whitespace().count();
    let char_count = input.chars().filter(|c| !c.is_whitespace()).count();
    let sentence_count = input.matches('.').count();

    format!("Text Analysis:\nWords: {}\nCharacters (excluding spaces): {}\nSentences: {}", 
            word_count, char_count, sentence_count)
}

pub fn password_generator() {
    let password: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    println!("Generated Password: {}", password);
}

pub fn calculator() {
    println!("Calculator: Enter an expression (e.g., 3 + 4):");
    let mut input = String::new();
    if let Ok(_) = io::stdin().read_line(&mut input) {
        let input = input.trim();
        match meval::eval_str(input) {
            Ok(result) => println!("Result: {}", result),
            Err(err) => println!("Error evaluating expression: {}", err),
        }
    }
}
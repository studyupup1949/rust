use std::env;
use std::process;

mod regex_generator; // Assuming your main logic is in `regex_generator.rs`
use regex_generator::RegexGenerator;

fn print_help() {
    println!(
        "Usage: regex_generator [OPTIONS] PATTERN [INCREMENT] [ARRAY]
    
    OPTIONS:
        -h, --help              Prints help information
        -v, --version           Prints version information
        -p, --pattern PATTERN   Specifies the pattern to use
        -i, --increment VALUE   Initial value for the increment (optional)
        -a, --array VALUE       Array of strings (comma-separated) for /a pattern (optional)
    
    PATTERN:
        The pattern to be used for generating the string.
    
    INCREMENT:
        The initial increment value, if needed. If not provided, default is 0.
    
    ARRAY:
        A comma-separated list of strings to be used with the /a pattern.
    
    SUPPORTED PATTERNS:

    Pattern       Description
    ----------------------------------------------------------------
    \\d           Any digit from 0 to 9
    \\w           Any word character (letters, digits, and underscore)
    \\s           Any whitespace character (space, tab, newline)
    \\D           Any character that is not a digit
    \\W           Any character that is not a word character
    \\S           Any character that is not a whitespace character
    \\t           Tab character
    \\n           Newline character
    \\i           Incrementing value (use with optional ｛:length｝ for leading zeros)
    \\a           Random string from an array (use with optional + or - for order)
    [abc]         Any one of the characters a, b, or c
    [a-z]         Any character in the range a to z
    [^a-z]        Any character not in the range a to z
    ｛n｝           Exactly n repetitions of the previous element
    ｛n,m｝         Between n and m repetitions of the previous element
    ｛n:m｝         Between n and m repetitions with leading zeros
    (abc)         Capture group for abc
    a|b           Alternation (matches either a or b)


    Example:
        regex_generator -p '\\i｛:10｝' -i 43
        regex_generator -p '[A-Za-z]｛5｝' -a 'apple,banana,grape'
"
    );
}

fn print_version() {
    println!("Regex Generator Version 1.0.0");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args.contains(&String::from("-h")) || args.contains(&String::from("--help")) {
        print_help();
        return;
    }

    if args.contains(&String::from("-v")) || args.contains(&String::from("--version")) {
        print_version();
        return;
    }

    let mut pattern = String::new();
    let mut increment_value: Option<String> = None;
    let mut array_values: Option<Vec<String>> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--pattern" => {
                if i + 1 < args.len() {
                    pattern = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: No pattern provided.");
                    process::exit(1);
                }
            }
            "-i" | "--increment" => {
                if i + 1 < args.len() {
                    increment_value = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: No increment value provided.");
                    process::exit(1);
                }
            }
            "-a" | "--array" => {
                if i + 1 < args.len() {
                    array_values = Some(args[i + 1].split(',').map(|s| s.to_string()).collect());
                    i += 1;
                } else {
                    eprintln!("Error: No array provided.");
                    process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option or missing value for {}", args[i]);
                process::exit(1);
            }
        }
        i += 1;
    }

    if pattern.is_empty() {
        eprintln!("Error: Pattern is required.");
        process::exit(1);
    }

    let mut generator = RegexGenerator::new(&pattern, increment_value, array_values);
    let result = generator.generate();
    println!("{}", result);
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_ascending() {
        let pattern = r"\i";
        let increment_value = Some("1299".to_string());
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        let expected_results = vec!["1300", "1301", "1302", "1303", "1304"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }
    }

    #[test]
    fn test_increment_descending() {
        let pattern = r"\i-";
        let increment_value = Some("1300".to_string());
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        let expected_results = vec!["1299", "1298", "1297", "1296", "1295"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }
    }

    #[test]
    fn test_array_random() {
        let pattern = r"\a";
        let increment_value = None;
        let array_values = Some(vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
        let mut generator = RegexGenerator::new(pattern, increment_value, array_values);

        for _ in 0..5 {
            let generated = generator.generate();
            assert!(["apple", "banana", "cherry"].contains(&generated.as_str()));
        }
    }

    #[test]
    fn test_array_ascending() {
        let pattern = r"\a+";
        let increment_value = None;
        let array_values = Some(vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
        let mut generator = RegexGenerator::new(pattern, increment_value, array_values);

        let expected_results = vec!["apple", "banana", "cherry", "apple", "banana"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }
    }

    #[test]
    fn test_array_descending() {
        let pattern = r"\a-";
        let increment_value = None;
        let array_values = Some(vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
        let mut generator = RegexGenerator::new(pattern, increment_value, array_values);

        let expected_results = vec!["cherry", "banana", "apple", "cherry", "banana"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }
    }

    #[test]
    fn test_combined_increment_and_array_ascending() {
        let pattern = r"\a+\i+";
        let increment_value = Some("1299".to_string());
        let array_values = Some(vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
        let mut generator = RegexGenerator::new(pattern, increment_value, array_values);

        let expected_results = vec!["apple1300", "banana1301", "cherry1302", "apple1303", "banana1304"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }
    }

    #[test]
    fn test_combined_increment_and_array_descending() {
        let pattern = r"\a-\i-";
        let increment_value = Some("1304".to_string());
        let array_values = Some(vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
        let mut generator = RegexGenerator::new(pattern, increment_value, array_values);

        let expected_results = vec!["cherry1303", "banana1302", "apple1301", "cherry1300", "banana1299"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }
    }

    #[test]
    fn test_character_classes() {
        let pattern = r"\d\w\s";
        let increment_value = None;
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        for _ in 0..5 {
            let generated = generator.generate();
            print!("{}", generated);
            assert!(generated.len() == 3);
            assert!(generated.chars().nth(0).unwrap().is_digit(10));
            assert!(generated.chars().nth(1).unwrap().is_alphanumeric());
            assert!(generated.chars().nth(2).unwrap().is_whitespace());
        }
    }

    #[test]
    fn test_custom_repeat() {
        let pattern = r"\d{2,4}";
        let increment_value = None;
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        for _ in 0..5 {
            let generated = generator.generate();
            assert!(generated.len() >= 2 && generated.len() <= 4);
            assert!(generated.chars().all(|c| c.is_digit(10)));
        }
    }

    #[test]
    fn test_character_ranges() {
        let pattern = r"[a-c]{3}";
        let increment_value = None;
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        for _ in 0..5 {
            let generated = generator.generate();
            assert!(generated.len() == 3);
            assert!(generated.chars().all(|c| c >= 'a' && c <= 'c'));
        }
    }

    #[test]
    fn test_character_negation() {
        let pattern = r"[^a-c]{3}";
        let increment_value = None;
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        for _ in 0..5 {
            let generated = generator.generate();
            assert!(generated.len() == 3);
            assert!(generated.chars().all(|c| c < 'a' || c > 'c'));
        }
    }

    #[test]
    fn test_group_capturing_and_backreference() {
        let pattern = r"(ab)\+(cd)=\2\+\1";
        let increment_value = None;
        let mut generator = RegexGenerator::new(pattern, increment_value, None);

        for _ in 0..5 {
            let generated = generator.generate();
            let parts: Vec<&str> = generated.split('=').collect();
            assert_eq!(parts.len(), 2);

            let left_side: Vec<&str> = parts[0].split('+').collect();
            let right_side: Vec<&str> = parts[1].split('+').collect();

            assert_eq!(left_side[1], right_side[0]);
            assert_eq!(left_side[0], right_side[1]);
        }
    }
    #[test]
    fn test_leading_zero(){
        let pattern: &str = r"[0-9]{3:10}";
        let increment_value = None;
        let mut generator = RegexGenerator::new(pattern, increment_value, None);
        for _ in 0..5 {
            let generated = generator.generate();
            assert!(generated.len() == 10);
            assert!(generated.chars().nth(6).unwrap() == '0');

        }

    }

    #[test]
    fn test_increment_leading_zero(){
        let pattern:&str = r"\i{:5}";
        let increment_value = Some("998".to_string());
        let mut generator = RegexGenerator::new(pattern, increment_value, None);
        let expected_results = vec!["00999", "01000", "01001", "01002", "01003"];
        for expected in expected_results {
            let generated = generator.generate();
            assert_eq!(generated, expected);
        }

    }



}


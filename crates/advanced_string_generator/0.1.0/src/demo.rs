use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct RegexGenerator {
    pattern: String,
    groups: HashMap<usize, String>,
    increment_value: Option<String>,
    direction: i32, // 1 for ascending, -1 for descending
    array_values: Option<Vec<String>>, // Optional array of strings
    array_index: usize, // Index to track ascending or descending order
}

impl RegexGenerator {
    pub fn new(pattern: &str, increment_value: Option<String>, array_values: Option<Vec<String>>) -> Self {
        Self {
            pattern: pattern.to_string(),
            groups: HashMap::new(),
            increment_value,
            direction: 1, // default to ascending
            array_values, // store the array of strings
            array_index: 0, // start at the beginning of the array
        }
    }

    pub fn generate(&mut self) -> String {
        let mut result = String::new();
        let mut chars = self.pattern.chars().peekable();
        let mut group_stack: Vec<String> = Vec::new();
        let mut current_group: Option<usize> = None;
        let mut group_index: usize = 1;

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        'i' => {
                            // Check for + or - sign
                            let sign = if chars.peek() == Some(&'+') {
                                chars.next();
                                1 // Ascending
                            } else if chars.peek() == Some(&'-') {
                                chars.next();
                                -1 // Descending
                            } else {
                                1 // Default to ascending
                            };

                            self.direction = sign;

                            if let Some(increment_value) = self.increment_value.take() {
                                let new_value = self.increment_string(&increment_value);
                                result.push_str(&new_value);
                                self.increment_value = Some(new_value);
                            } else {
                                result.push_str("0"); // Default to "0" or another placeholder
                            }
                        }
                        'a' => {
                            let array_sign = if chars.peek() == Some(&'+') {
                                chars.next();
                                1 // Ascending
                            } else if chars.peek() == Some(&'-') {
                                chars.next();
                                -1 // Descending
                            } else {
                                0 // Random
                            };

                            if let Some(ref array) = self.array_values {
                                match array_sign {
                                    1 => {
                                        // Ascending order
                                        let value = &array[self.array_index % array.len()];
                                        result.push_str(value);
                                        self.array_index += 1;
                                    }
                                    -1 => {
                                        // Descending order
                                        let index = array.len() - 1 - (self.array_index % array.len());
                                        let value = &array[index];
                                        result.push_str(value);
                                        self.array_index += 1;
                                    }
                                    _ => {
                                        // Random order
                                        let mut rng = rand::thread_rng();
                                        let random_string = &array[rng.gen_range(0..array.len())];
                                        result.push_str(random_string);
                                    }
                                }
                            } else {
                                result.push_str(""); // If no array is provided, insert nothing or handle as needed
                            }
                        }
                        '1'..='9' => {
                            if let Some(content) = self.groups.get(&(next_ch.to_digit(10).unwrap() as usize)) {
                                result.push_str(content);
                            }
                        }
                        _ => {
                            if let Some(repeat_spec) = self.check_repeat_spec(&mut chars) {
                                result.push_str(&self.handle_repeat(next_ch, repeat_spec));
                            } else {
                                result.push_str(&self.handle_escape(next_ch));
                            }
                        }
                    }
                }
            } else if ch == '[' {
                let (char_class, negate) = self.extract_char_class(&mut chars);
                if let Some(repeat_spec) = self.check_repeat_spec(&mut chars) {
                    result.push_str(&self.handle_bracket(char_class, repeat_spec, negate));
                } else {
                    result.push_str(&self.handle_bracket(char_class, (1, None), negate));
                }
            } else if ch == '(' {
                if chars.peek() == Some(&'?') {
                    chars.next(); // Skip the '?'
                    // Handle non-capturing groups or other special groups here
                }
                current_group = Some(group_index);
                group_stack.push(String::new());
                group_index += 1;
            } else if ch == ')' {
                if let Some(group) = current_group {
                    if let Some(mut content) = group_stack.pop() {
                        if let Some(_alt_pos) = content.find('|') {
                            let choices: Vec<&str> = content.split('|').collect();
                            content = choices[0].to_string();
                        }
                        self.groups.insert(group, content.clone());
                        result.push_str(&content);
                        current_group = None;
                    }
                }
            } else if ch == '|' {
                if let Some(last) = group_stack.last_mut() {
                    last.push('|');
                } else {
                    result.push('|');
                }
            } else {
                if let Some(ref mut _current) = current_group {
                    if let Some(last) = group_stack.last_mut() {
                        last.push(ch);
                    }
                } else {
                    result.push(ch);
                }
            }
        }

        result
    }

    fn check_repeat_spec<I>(&self, chars: &mut std::iter::Peekable<I>) -> Option<(usize, Option<usize>)>
    where
        I: Iterator<Item = char>,
    {
        if chars.peek() == Some(&'{') {
            chars.next(); // Skip the '{'
            let mut spec = String::new();

            while let Some(&c) = chars.peek() {
                if c == '}' {
                    chars.next(); // Skip the '}'
                    break;
                }
                spec.push(c);
                chars.next();
            }

            let parts: Vec<&str> = spec.split(',').collect();
            if parts.len() == 1 {
                return Some((parts[0].parse().unwrap(), None));
            } else if parts.len() == 2 {
                return Some((parts[0].parse().unwrap(), Some(parts[1].parse().unwrap())));
            }
        }

        None
    }

    fn handle_escape(&self, ch: char) -> String {
        let mut rng = rand::thread_rng();

        match ch {
            'd' => rng.gen_range(0..10).to_string(), // \d - any digit
            'w' => {
                let sample_set = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";
                sample_set.chars().nth(rng.gen_range(0..sample_set.len())).unwrap().to_string()
            } // \w - any word character
            's' => {
                let sample_set = " \t\n\r";
                sample_set.chars().nth(rng.gen_range(0..sample_set.len())).unwrap().to_string()
            } // \s - any whitespace
            'D' => {
                let sample_set = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()";
                sample_set.chars().nth(rng.gen_range(0..sample_set.len())).unwrap().to_string()
            } // \D - any non-digit character
            'W' => {
                let sample_set = "!@#$%^&*()+=-[]{}|;:,.<>?/`~";
                sample_set.chars().nth(rng.gen_range(0..sample_set.len())).unwrap().to_string()
            } // \W - any non-word character
            'S' => {
                let sample_set = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()";
                sample_set.chars().nth(rng.gen_range(0..sample_set.len())).unwrap().to_string()
            } // \S - any non-whitespace character
            't' => "\t".to_string(), // \t - Tab character
            'n' => "\n".to_string(), // \n - Line feed character
            _ => ch.to_string(),
        }
    }

    fn handle_repeat(&self, ch: char, repeat_spec: (usize, Option<usize>)) -> String {
        let (min, max) = repeat_spec;
        let mut rng = rand::thread_rng();
        let repeat_count = if let Some(max) = max {
            rng.gen_range(min..=max)
        } else {
            min
        };

        std::iter::repeat(self.handle_escape(ch))
            .take(repeat_count)
            .collect()
    }

    fn extract_char_class<I>(&self, chars: &mut std::iter::Peekable<I>) -> (HashSet<char>, bool)
    where
        I: Iterator<Item = char>,
    {
        let mut char_class = HashSet::new();
        let mut negate = false;
        let mut range_start = None;

        if chars.peek() == Some(&'^') {
            chars.next();
            negate = true;
        }

        while let Some(ch) = chars.next() {
            if ch == ']' {
                break;
            } else if ch == '-' && range_start.is_some() {
                if let Some(range_end) = chars.next() {
                    let start = range_start.unwrap();
                    for c in start..=range_end {
                        char_class.insert(c);
                    }
                    range_start = None;
                }
            } else {
                range_start = Some(ch);
                char_class.insert(ch);
            }
        }

        (char_class, negate)
    }

    fn handle_bracket(&self, char_class: HashSet<char>, repeat_spec: (usize, Option<usize>), negate: bool) -> String {
        let (min, max) = repeat_spec;
        let mut rng = rand::thread_rng();
        let repeat_count = if let Some(max) = max {
            rng.gen_range(min..=max)
        } else {
            min
        };

        let sample_set: Vec<char> = if negate {
            let full_set: HashSet<char> = (32..127).map(|c| c as u8 as char).collect();
            full_set.difference(&char_class).cloned().collect()
        } else {
            char_class.into_iter().collect()
        };

        (0..repeat_count)
            .map(|_| sample_set[rng.gen_range(0..sample_set.len())])
            .collect()
    }

    fn increment_string(&self, value: &str) -> String {
        let mut prefix = String::new();
        let mut digits = String::new();

        // Separate prefix and numeric part
        for ch in value.chars() {
            if ch.is_digit(10) {
                digits.push(ch);
            } else {
                if digits.is_empty() {
                    prefix.push(ch);
                } else {
                    break;
                }
            }
        }

        // Adjust numeric part based on the direction (ascending or descending)
        if let Ok(num) = digits.parse::<i32>() {
            let adjusted_num = num + self.direction;
            digits = format!("{:0width$}", adjusted_num, width = digits.len());
        }

        // Combine prefix and adjusted numeric part
        format!("{}{}", prefix, digits)
    }
}

fn main() {
    let pattern = r"\a\d\d";
    let increment_value = Some("1299".to_string());
    let array_values = Some(vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
    let mut generator = RegexGenerator::new(pattern, increment_value, array_values);

    for _ in 0..5 {
        let generated = generator.generate();
        println!("{}", generated);
    }
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
}

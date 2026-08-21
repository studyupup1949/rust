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

                            // Check for leading zero specifier {:total_len}
                            
                            let total_len = if chars.peek() == Some(&'{') {
                                chars.next(); // Skip the '{'
                                let mut spec = String::new();
                                while let Some(&c) = chars.peek() {
                                    if c == '}' {
                                        chars.next(); // Skip the '}'
                                        break;
                                    }
                                    if c != ':' && c.is_numeric() {
                                        spec.push(c);
                                    }
                                    chars.next();
                                }
                                spec.parse::<usize>().ok()
                            } else {
                                None
                            };

                            if let Some(increment_value) = self.increment_value.take() {
                                let new_value = self.increment_string(&increment_value, total_len);
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
                    result.push_str(&self.handle_bracket(char_class, (1, None, None), negate));
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

    fn check_repeat_spec<I>(&self, chars: &mut std::iter::Peekable<I>) -> Option<(usize, Option<usize>, Option<(usize, usize)>)>
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

            if let Some(colon_pos) = spec.find(':') {
                // Handle leading zeros pattern {num_len:total_len}
                let num_len = spec[..colon_pos].parse().ok()?;
                let total_len = spec[colon_pos + 1..].parse().ok()?;
                return Some((1, None, Some((num_len, total_len))));
            } else {
                // Handle regular repeat pattern {min,max}
                let parts: Vec<&str> = spec.split(',').collect();
                if parts.len() == 1 {
                    return Some((parts[0].parse().unwrap(), None, None));
                } else if parts.len() == 2 {
                    return Some((parts[0].parse().unwrap(), Some(parts[1].parse().unwrap()), None));
                }
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

    fn handle_repeat(&self, ch: char, repeat_spec: (usize, Option<usize>, Option<(usize, usize)>)) -> String {
        let (min, max, leading_zeros_spec) = repeat_spec;
        let mut rng = rand::thread_rng();
        let repeat_count = if let Some(max) = max {
            rng.gen_range(min..=max)
        } else {
            min
        };

        if let Some((num_len, total_len)) = leading_zeros_spec {
            // Handle leading zeros pattern
            let number = rng.gen_range(10_usize.pow((num_len - 1) as u32)..10_usize.pow(num_len as u32));
            return format!("{:0width$}", number, width = total_len);
        } else {
            // Handle regular repeat pattern
            return std::iter::repeat(self.handle_escape(ch))
                .take(repeat_count)
                .collect();
        }
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

    fn handle_bracket(&self, char_class: HashSet<char>, repeat_spec: (usize, Option<usize>, Option<(usize, usize)>), negate: bool) -> String {
        let (min, max, leading_zeros_spec) = repeat_spec;
        let mut rng = rand::thread_rng();
        let repeat_count = if let Some(max) = max {
            rng.gen_range(min..=max)
        } else {
            min
        };

        if let Some((num_len, total_len)) = leading_zeros_spec {
            // Handle leading zeros pattern
            let number = rng.gen_range(10_usize.pow((num_len - 1) as u32)..10_usize.pow(num_len as u32));
            return format!("{:0width$}", number, width = total_len);
        } else {
            let sample_set: Vec<char> = if negate {
                let full_set: HashSet<char> = (32..127).map(|c| c as u8 as char).collect();
                full_set.difference(&char_class).cloned().collect()
            } else {
                char_class.into_iter().collect()
            };

            return (0..repeat_count)
                .map(|_| sample_set[rng.gen_range(0..sample_set.len())])
                .collect();
        }
    }

    fn increment_string(&self, value: &str, total_len: Option<usize>) -> String {
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
            digits = if let Some(total_len) = total_len {
                format!("{:0width$}", adjusted_num, width = total_len)
            } else {
                format!("{}", adjusted_num)
            };
        }

        // Combine prefix and adjusted numeric part
        format!("{}{}", prefix, digits)
    }
}

use crate::abi::{AbiInput, AbiItem, AbiOutput};
use std::collections::HashMap;

pub struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse_abi(&mut self) -> Result<Vec<AbiItem>, String> {
        self.skip_whitespace();

        if self.current() == Some('[') {
            self.parse_abi_array()
        } else if self.current() == Some('{') {
            let obj = self.parse_object()?;
            if let Some(Value::Array(arr)) = obj.get("abi") {
                self.convert_to_abi_items(arr)
            } else {
                Err("Expected 'abi' field in object".to_string())
            }
        } else {
            Err("Expected JSON array or object".to_string())
        }
    }

    fn parse_abi_array(&mut self) -> Result<Vec<AbiItem>, String> {
        let arr = self.parse_array()?;
        self.convert_to_abi_items(&arr)
    }

    fn convert_to_abi_items(&self, arr: &[Value]) -> Result<Vec<AbiItem>, String> {
        let mut items = Vec::new();

        for value in arr {
            if let Value::Object(obj) = value {
                if let Some(item) = self.convert_to_abi_item(obj) {
                    items.push(item);
                }
            }
        }

        Ok(items)
    }

    fn convert_to_abi_item(&self, obj: &HashMap<String, Value>) -> Option<AbiItem> {
        let r#type = obj.get("type")?.as_string()?;

        Some(AbiItem {
            r#type,
            name: obj.get("name").and_then(|v| v.as_string()),
            inputs: obj.get("inputs").and_then(|v| self.convert_to_inputs(v)),
            outputs: obj.get("outputs").and_then(|v| self.convert_to_outputs(v)),
            state_mutability: obj.get("stateMutability").and_then(|v| v.as_string()),
            anonymous: obj.get("anonymous").and_then(|v| v.as_bool()),
            payable: obj.get("payable").and_then(|v| v.as_bool()),
            constant: obj.get("constant").and_then(|v| v.as_bool()),
        })
    }

    fn convert_to_inputs(&self, value: &Value) -> Option<Vec<AbiInput>> {
        if let Value::Array(arr) = value {
            let mut inputs = Vec::new();
            for item in arr {
                if let Value::Object(obj) = item {
                    if let Some(input) = self.convert_to_input(obj) {
                        inputs.push(input);
                    }
                }
            }
            Some(inputs)
        } else {
            None
        }
    }

    fn convert_to_input(&self, obj: &HashMap<String, Value>) -> Option<AbiInput> {
        let r#type = obj.get("type")?.as_string()?;

        Some(AbiInput {
            name: obj.get("name").and_then(|v| v.as_string()),
            r#type,
            indexed: obj.get("indexed").and_then(|v| v.as_bool()),
            internal_type: obj.get("internalType").and_then(|v| v.as_string()),
            components: obj
                .get("components")
                .and_then(|v| self.convert_to_inputs(v)),
        })
    }

    fn convert_to_outputs(&self, value: &Value) -> Option<Vec<AbiOutput>> {
        if let Value::Array(arr) = value {
            let mut outputs = Vec::new();
            for item in arr {
                if let Value::Object(obj) = item {
                    if let Some(output) = self.convert_to_output(obj) {
                        outputs.push(output);
                    }
                }
            }
            Some(outputs)
        } else {
            None
        }
    }

    fn convert_to_output(&self, obj: &HashMap<String, Value>) -> Option<AbiOutput> {
        let r#type = obj.get("type")?.as_string()?;

        Some(AbiOutput {
            name: obj.get("name").and_then(|v| v.as_string()),
            r#type,
            internal_type: obj.get("internalType").and_then(|v| v.as_string()),
            components: obj
                .get("components")
                .and_then(|v| self.convert_to_outputs(v)),
        })
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();

        match self.current() {
            Some('"') => self.parse_string(),
            Some('[') => Ok(Value::Array(self.parse_array()?)),
            Some('{') => Ok(Value::Object(self.parse_object()?)),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            Some(c) if c.is_ascii_digit() || c == '-' => self.parse_number(),
            _ => Err("Unexpected character".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<Vec<Value>, String> {
        self.expect('[')?;
        let mut arr = Vec::new();

        self.skip_whitespace();
        if self.current() == Some(']') {
            self.advance();
            return Ok(arr);
        }

        loop {
            arr.push(self.parse_value()?);
            self.skip_whitespace();

            if self.current() == Some(',') {
                self.advance();
                self.skip_whitespace();
            } else if self.current() == Some(']') {
                self.advance();
                break;
            } else {
                return Err("Expected ',' or ']' in array".to_string());
            }
        }

        Ok(arr)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, Value>, String> {
        self.expect('{')?;
        let mut obj = HashMap::new();

        self.skip_whitespace();
        if self.current() == Some('}') {
            self.advance();
            return Ok(obj);
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                Value::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };

            self.skip_whitespace();
            self.expect(':')?;

            let value = self.parse_value()?;
            obj.insert(key, value);

            self.skip_whitespace();
            if self.current() == Some(',') {
                self.advance();
            } else if self.current() == Some('}') {
                self.advance();
                break;
            } else {
                return Err("Expected ',' or '}' in object".to_string());
            }
        }

        Ok(obj)
    }

    fn parse_string(&mut self) -> Result<Value, String> {
        self.expect('"')?;
        let mut string = String::new();

        while self.position < self.input.len() {
            match self.current() {
                Some('"') => {
                    self.advance();
                    return Ok(Value::String(string));
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => string.push('\n'),
                        Some('r') => string.push('\r'),
                        Some('t') => string.push('\t'),
                        Some('\\') => string.push('\\'),
                        Some('"') => string.push('"'),
                        Some('u') => {
                            self.advance();
                            let hex: String = (0..4)
                                .filter_map(|_| {
                                    let c = self.current()?;
                                    self.advance();
                                    Some(c)
                                })
                                .collect();
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(ch) = char::from_u32(code) {
                                    string.push(ch);
                                }
                            }
                            continue;
                        }
                        Some(c) => string.push(c),
                        None => return Err("Unexpected end of string".to_string()),
                    }
                    self.advance();
                }
                Some(c) => {
                    string.push(c);
                    self.advance();
                }
                None => return Err("Unterminated string".to_string()),
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let mut num_str = String::new();

        if self.current() == Some('-') {
            num_str.push('-');
            self.advance();
        }

        while let Some(c) = self.current() {
            if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
            num_str
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| "Invalid number".to_string())
        } else {
            num_str
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| "Invalid number".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<Value, String> {
        if self.consume_word("true") {
            Ok(Value::Bool(true))
        } else if self.consume_word("false") {
            Ok(Value::Bool(false))
        } else {
            Err("Invalid boolean".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<Value, String> {
        if self.consume_word("null") {
            Ok(Value::Null)
        } else {
            Err("Invalid null".to_string())
        }
    }

    fn consume_word(&mut self, word: &str) -> bool {
        let chars: Vec<char> = word.chars().collect();
        let start = self.position;

        for expected in chars {
            if self.current() != Some(expected) {
                self.position = start;
                return false;
            }
            self.advance();
        }

        true
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.current() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected '{expected}'"))
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

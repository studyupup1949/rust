use crate::aaml::parsing;
use crate::error::AamlError;
use crate::types_aam::list::ListType;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::collections::HashMap;

pub struct AamDeserializer {
    input: String,
}

impl AamDeserializer {
    fn new(input: String) -> Self {
        Self { input }
    }

    fn parse_fields(&self) -> Result<HashMap<String, String>, AamlError> {
        let v = self.input.trim();
        if v.starts_with('{') && v.ends_with('}') {
            let pairs = parsing::parse_inline_object(v)?;
            return Ok(pairs.into_iter().collect());
        }
        let mut fields = HashMap::new();
        let mut lines = self.input.lines().peekable();
        while let Some(line) = lines.next() {
            let stripped = parsing::strip_comment(line).trim();
            if stripped.is_empty() || stripped.starts_with('@') {
                continue;
            }
            let mut accumulated = stripped.to_string();
            while !is_balanced(&accumulated) {
                match lines.next() {
                    Some(next) => {
                        let next_stripped = parsing::strip_comment(next).trim();
                        accumulated.push(' ');
                        accumulated.push_str(next_stripped);
                    }
                    None => break,
                }
            }
            if let Some(pos) = accumulated.find('=') {
                let key = accumulated[..pos].trim().to_string();
                let val = accumulated[pos + 1..].trim().to_string();
                if !key.is_empty() {
                    fields.insert(key, val);
                }
            }
        }
        Ok(fields)
    }
}

fn is_balanced(s: &str) -> bool {
    let mut depth: i32 = 0;
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        match ch {
            '"' | '\'' => match in_quote {
                Some(q) if q == ch => in_quote = None,
                None => in_quote = Some(ch),
                _ => {}
            },
            '{' | '[' if in_quote.is_none() => depth += 1,
            '}' | ']' if in_quote.is_none() => depth -= 1,
            _ => {}
        }
    }
    depth == 0 && in_quote.is_none()
}

pub fn from_str<'de, T: serde::Deserialize<'de>>(s: &'de str) -> Result<T, AamlError> {
    let mut deserializer = AamDeserializer::new(s.to_string());
    T::deserialize(&mut deserializer)
}

macro_rules! impl_deserialize_primitive {
    ($deserialize:ident, $visit:ident, $ty:ty) => {
        fn $deserialize<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            self.input
                .trim()
                .parse::<$ty>()
                .map_err(|e| AamlError::InvalidValue {
                    details: format!(
                        "cannot parse '{}' as {}: {}",
                        self.input,
                        stringify!($ty),
                        e
                    ),
                    expected: format!("valid {}", stringify!($ty)),
                    diagnostics: None,
                })
                .and_then(|v| visitor.$visit(v))
        }
    };
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut AamDeserializer {
    type Error = AamlError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = self.input.trim();
        if v.starts_with('{') && v.ends_with('}') {
            return self.deserialize_map(visitor);
        }
        if v.starts_with('[') && v.ends_with(']') {
            return self.deserialize_seq(visitor);
        }
        if v == "true" {
            return visitor.visit_bool(true);
        }
        if v == "false" {
            return visitor.visit_bool(false);
        }
        if let Ok(n) = v.parse::<i64>() {
            return visitor.visit_i64(n);
        }
        if let Ok(n) = v.parse::<f64>() {
            return visitor.visit_f64(n);
        }
        visitor.visit_str(v)
    }

    impl_deserialize_primitive!(deserialize_bool, visit_bool, bool);
    impl_deserialize_primitive!(deserialize_i8, visit_i8, i8);
    impl_deserialize_primitive!(deserialize_i16, visit_i16, i16);
    impl_deserialize_primitive!(deserialize_i32, visit_i32, i32);
    impl_deserialize_primitive!(deserialize_i64, visit_i64, i64);
    impl_deserialize_primitive!(deserialize_u8, visit_u8, u8);
    impl_deserialize_primitive!(deserialize_u16, visit_u16, u16);
    impl_deserialize_primitive!(deserialize_u32, visit_u32, u32);
    impl_deserialize_primitive!(deserialize_u64, visit_u64, u64);
    impl_deserialize_primitive!(deserialize_f32, visit_f32, f32);
    impl_deserialize_primitive!(deserialize_f64, visit_f64, f64);

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_str(self.input.trim())
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_string(self.input.trim().to_string())
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(AamlError::InvalidValue {
            details: "bytes deserialization not supported".to_string(),
            expected: "string value".to_string(),
            diagnostics: None,
        })
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(AamlError::InvalidValue {
            details: "byte_buf deserialization not supported".to_string(),
            expected: "string value".to_string(),
            diagnostics: None,
        })
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = self.input.trim();
        if v.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = self.input.trim();
        let items = ListType::parse_items(v).ok_or_else(|| AamlError::InvalidValue {
            details: format!("'{v}' is not a valid list literal"),
            expected: "[item, item, ...] format".to_string(),
            diagnostics: None,
        })?;
        visitor.visit_seq(ListSeq::new(items))
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let fields = self.parse_fields()?;
        visitor.visit_map(FieldMapAccess {
            iter: fields.into_iter(),
            current_value: None,
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(AamlError::InvalidValue {
            details: "enum deserialization not supported".to_string(),
            expected: "key = value form".to_string(),
            diagnostics: None,
        })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

struct ListSeq {
    items: Vec<String>,
    index: usize,
}

impl ListSeq {
    fn new(items: Vec<String>) -> Self {
        Self { items, index: 0 }
    }
}

impl<'de> SeqAccess<'de> for ListSeq {
    type Error = AamlError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        if self.index >= self.items.len() {
            return Ok(None);
        }
        let item = std::mem::take(&mut self.items[self.index]);
        self.index += 1;
        let mut de = AamDeserializer::new(item);
        seed.deserialize(&mut de).map(Some)
    }
}

struct FieldMapAccess {
    iter: std::collections::hash_map::IntoIter<String, String>,
    current_value: Option<String>,
}

impl<'de> MapAccess<'de> for FieldMapAccess {
    type Error = AamlError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.current_value = Some(value);
                let mut key_de = AamDeserializer::new(key);
                seed.deserialize(&mut key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let value = self
            .current_value
            .take()
            .expect("next_value_seed called before next_key_seed");
        let mut de = AamDeserializer::new(value);
        seed.deserialize(&mut de)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_primitives() {
        let mut de = AamDeserializer::new("42".to_string());
        let v: i32 = serde::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn test_deserialize_struct_full_aam() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Server {
            host: String,
            port: i32,
        }

        let s: Server = from_str("host = localhost\nport = 8080\n").unwrap();
        assert_eq!(s.host, "localhost");
        assert_eq!(s.port, 8080);
    }

    #[test]
    fn test_deserialize_struct_inline() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Server {
            host: String,
            port: i32,
        }

        let s: Server = from_str("{ host = localhost, port = 8080 }").unwrap();
        assert_eq!(s.host, "localhost");
        assert_eq!(s.port, 8080);
    }

    #[test]
    fn test_deserialize_option() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Server {
            host: String,
            debug: Option<bool>,
        }

        let s: Server = from_str("host = localhost\n").unwrap();
        assert_eq!(s.host, "localhost");
        assert_eq!(s.debug, None);

        let s2: Server = from_str("host = localhost\ndebug = true\n").unwrap();
        assert_eq!(s2.debug, Some(true));
    }

    #[test]
    fn test_deserialize_vec() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct App {
            name: String,
            tags: Vec<String>,
            ports: Vec<i32>,
        }

        let app: App = from_str("name = MyApp\ntags = [rust, aam]\nports = [80, 443]\n").unwrap();
        assert_eq!(app.name, "MyApp");
        assert_eq!(app.tags, vec!["rust", "aam"]);
        assert_eq!(app.ports, vec![80, 443]);
    }

    #[test]
    fn test_deserialize_nested() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Address {
            host: String,
            port: i32,
        }

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Server {
            name: String,
            address: Address,
        }

        let s: Server =
            from_str("name = proxy\naddress = { host = 10.0.0.1, port = 3128 }\n").unwrap();
        assert_eq!(s.name, "proxy");
        assert_eq!(s.address.host, "10.0.0.1");
        assert_eq!(s.address.port, 3128);
    }

    #[test]
    fn test_deserialize_list_of_structs() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Item {
            name: String,
            weight: f64,
        }

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Chest {
            title: String,
            loot: Vec<Item>,
        }

        let chest: Chest = from_str(
            r#"
            title = Golden Chest
            loot = [
                { name = Coin, weight = 0.1 },
                { name = Gem, weight = 0.3 }
            ]
            "#,
        )
        .unwrap();
        assert_eq!(chest.title, "Golden Chest");
        assert_eq!(chest.loot.len(), 2);
        assert_eq!(chest.loot[1].name, "Gem");
    }
}

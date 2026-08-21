//! Library to read and write ADIF formated files
//!
//! licensed under CC BY-SA 4.0,
//! To view a copy of this license, visit <http://creativecommons.org/licenses/by-sa/4.0/>
//!
//! Author: Andreas, <df1asc@darc.de>
//!
//! # Example
//!
// This example does not print anything if run as a doctest
//! ```
//! use std::fs;
//! use adif_io::{DeserializeADI, Doc};
//!
//! fn main() {
//!     let content = fs::read_to_string("test_data/big_testfile_1000.adi").expect("error reading ADI file: {err}");
//!     let mut doc = Doc::new();
//!     doc.deserialize_adi(&content).expect("could not deserialize from ADI");
//!
//!     // Header info from file
//!     let header = doc.header();
//!     println!("Comment  : {}", header.comment());
//!     println!("Prog ID  : {}", header.program_id());
//!     println!("Prog Ver : {}", header.program_ver());
//!
//!     // Count QSOs and print them
//!     println!("QSO count: {}", doc.iter_record().count());
//!     doc.iter_record().enumerate().for_each(|(i, qso)| println!("QSO {}: {:?}", i+1, qso));
//!
//!     // Get a QSO and modify data
//!     let mut qso = doc.get_record_mut(5).expect("no QSO available");
//!     qso.insert("NOTES", "Test data".into());
//!
//!     // Create a `Record` and add it, case for field names does not matter
//!     let qso = Record::from(vec![
//!         ("QSO_DATE", "20231009"),
//!         ("TIME_ON", "1245"),
//!         ("Call", "DK5XXX"),
//!         ("NAME", "Chris"),  // Upper case field name inserted
//!     ]);
//!     println!("Name: {:?}", qso.get("name").unwrap());  // Accessed field name with lower case
//!     doc.add_record(qso);
//! }
//! ```

#[cfg(all(feature = "serde_impl", feature = "serde_loose"))]
compile_error!("feature 'serde_impl' and 'serde_loose' cannot be enabled at the same time");

use linked_hash_map::{Keys, LinkedHashMap};
use regex::Regex;
#[cfg(any(feature = "serde_impl", feature = "serde_loose"))]
use serde;
#[cfg(feature = "serde_loose")]
use serde::de::{Error, Visitor};
use serde_json::Value;
use std::collections::HashMap;
#[cfg(feature = "serde_loose")]
use std::fmt::Formatter;
use std::slice::Iter;
use std::sync::LazyLock;

/// ADIF type map for type collation of known fields
static TYPE_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let type_map = include_str!(concat!(env!("OUT_DIR"), "/type_map.json"));
    let type_map: Value = serde_json::from_str(type_map).expect("could not interpret JSON");
    let mut map: HashMap<String, String> = HashMap::new();
    if let Some(types) = type_map.as_object() {
        types
            .iter()
            .map(|(k, v)| {
                (
                    k,
                    if let Some(t) = v.as_str() {
                        t.to_string()
                    } else {
                        panic!("could not get 'Data Type' for '{}'", k)
                    },
                )
            })
            .for_each(|(k, v)| {
                let _ = map.insert(k.to_string(), v);
            });
        map
    } else {
        panic!("could not get 'Records' object")
    }
});

static RE_HEADER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[eE][oO][hH]>").unwrap());
static RE_RECORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[eE][oO][rR]>").unwrap());

const WRAP_AT: usize = 5;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Converting QSO data struct to ADI `String`
pub trait SerializeADI {
    /// Serialize ADIF data to ADIF formated String
    fn serialize_adi(&self) -> String;
}

/// Converting QSO data from an ADI `String`
pub trait DeserializeADI {
    /// Deserialize ADIF data from ADIF formated String
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), String>;
}

/// Test if all chars in the string are digits
fn str_is_digits(value: impl AsRef<str>) -> bool {
    value
        .as_ref()
        .chars()
        .filter(|c| !c.is_ascii_digit())
        .count()
        == 0
}

/// The ADIF types for ADI
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_impl", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    String(String),
    Digit(u8),
    Integer(i64),
    PositiveInteger(u64),
    Number(f64),
    // Character(char),
    Boolean(bool),
    Date(String),
    Time(String),
    GridSquare(String),
}

impl Type {
    /// Collate the value type from the field name if possible or fall back to `String`
    pub fn collate(field: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let value = value.as_ref();
        if let Some(t) = TYPE_MAP.get(&field.as_ref().to_uppercase()) {
            match t.as_str() {
                "String" => Type::String(value.to_string()),
                "Number" => {
                    if let Ok(v) = value.parse::<f64>() {
                        Type::Number(v)
                    } else {
                        Type::String(value.to_string())
                    }
                }
                "Boolean" => {
                    if value.to_uppercase() == "Y" {
                        Type::Boolean(true)
                    } else {
                        Type::Boolean(false)
                    }
                }
                "Date" => {
                    if value.len() == 8 && str_is_digits(value) {
                        Type::Date(format!("{}-{}-{}", &value[..4], &value[4..6], &value[6..8]))
                    } else {
                        Type::String(value.to_string())
                    }
                }
                "Time" => {
                    if (value.len() == 4 || value.len() == 6) && str_is_digits(value) {
                        Type::Time(
                            value
                                .chars()
                                .collect::<Vec<char>>()
                                .chunks(2)
                                .map(|c| c.iter().collect())
                                .collect::<Vec<String>>()
                                .join(":"),
                        )
                    } else {
                        Type::String(value.to_string())
                    }
                }
                "GridSquare" => {
                    if value.len() % 2 == 0 && !value.is_empty() && value.len() <= 8 {
                        Type::GridSquare(value.to_string()) // Todo: proper check and format
                    } else {
                        Type::String(value.to_string())
                    }
                }
                _ => Type::String(value.to_string()),
            }
        } else {
            Type::String(value.to_string())
        }
    }
}

impl From<&Type> for String {
    fn from(value: &Type) -> Self {
        match value {
            Type::String(v) => v.clone(),
            Type::Digit(v) => format!("{v}"),
            Type::Integer(v) => format!("{v}"),
            Type::PositiveInteger(v) => format!("{v}"),
            Type::Number(v) => format!("{v}"),
            // Type::Character(v) => format!("{v}"),
            Type::Boolean(v) => {
                if *v {
                    "Y".to_string()
                } else {
                    "N".to_string()
                }
            }
            Type::Date(v) => v.replace("-", ""),
            Type::Time(v) => v.replace(":", ""),
            Type::GridSquare(v) => v.clone(),
        }
    }
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        Type::String(value.to_string())
    }
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        Type::String(value)
    }
}

impl From<bool> for Type {
    fn from(value: bool) -> Self {
        Type::Boolean(value)
    }
}

impl SerializeADI for Type {
    fn serialize_adi(&self) -> String {
        self.into()
    }
}

#[cfg(feature = "serde_loose")]
impl serde::Serialize for Type {
    /// Serialize `Type` as close as possible
    ///
    /// to types: bool, f64, u8, u64 and everything else as String
    // Stripping of the `Type` overhead for a nicer representation
    // at the cost of not so well typed deserialization.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Type::Boolean(v) => serializer.serialize_bool(*v),
            Type::Number(v) => serializer.serialize_f64(*v),
            Type::Digit(v) => serializer.serialize_u8(*v),
            Type::Integer(v) => serializer.serialize_i64(*v),
            Type::PositiveInteger(v) => serializer.serialize_u64(*v),
            _ => {
                let value: String = self.into();
                serializer.serialize_str(&value)
            }
        }
    }
}

#[cfg(feature = "serde_loose")]
struct TypeVisitor;

#[cfg(feature = "serde_loose")]
impl Visitor<'_> for TypeVisitor {
    type Value = Type;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("ADIF compatible data")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Type::Boolean(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Type::Integer(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Type::PositiveInteger(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Type::Number(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Type::String(v.to_string()))
    }
}

#[cfg(feature = "serde_loose")]
impl<'de> serde::Deserialize<'de> for Type {
    /// Deserialize for `Type` as close as possible
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(TypeVisitor)
    }
}

/// Represents a single ADIF QSO record
///
/// The field name will be stored and handled as uppercase
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "serde_impl", feature = "serde_loose"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Record(LinkedHashMap<String, Type>);

impl Record {
    /// Create an empty ADIF QSO record.
    pub fn new() -> Self {
        Self(LinkedHashMap::new())
    }

    /// Add or change a field to a QSO record.
    pub fn insert(&mut self, field: impl AsRef<str>, val: Type) -> Option<Type> {
        self.0.insert(field.as_ref().to_uppercase(), val)
    }

    /// Check if a field is in the record.
    pub fn contains(&self, field: impl AsRef<str>) -> bool {
        self.0.contains_key(&field.as_ref().to_uppercase())
    }

    /// Get the field content.
    pub fn get(&self, field: impl AsRef<str>) -> Option<&Type> {
        self.0.get(&field.as_ref().to_uppercase())
    }

    /// Get the mutable field content.
    pub fn get_mut(&mut self, field: impl AsRef<str>) -> Option<&mut Type> {
        self.0.get_mut(&field.as_ref().to_uppercase())
    }

    /// Returns a double-ended iterator visiting all field-value pairs in order of insertion.
    pub fn iter(&'_ self) -> linked_hash_map::Iter<'_, String, Type> {
        self.0.iter()
    }

    /// Removes and returns the value corresponding to the field name from the record.
    pub fn remove(&mut self, field: impl AsRef<str>) -> Option<Type> {
        self.0.remove(&field.as_ref().to_uppercase())
    }

    /// Returns a double-ended iterator visiting all fields in order of insertion.
    pub fn keys(&self) -> Keys<'_, String, Type> {
        self.0.keys()
    }

    /// Builds a unique ID for a QSO.
    ///
    /// The QSO ID is generated from date, time, call and band.
    fn qso_id(&self) -> String {
        let mut id: String = self
            .get("QSO_DATE")
            .unwrap_or(&Type::String(String::from("#DDDDDD#")))
            .into();
        let time: String = self
            .get("TIME_ON")
            .unwrap_or(&Type::String(String::from("#TTTT#")))
            .into();
        let call: String = self
            .get("CALL")
            .unwrap_or(&Type::String(String::from("#CCCC#")))
            .into();
        let band: String = self
            .get("BAND")
            .unwrap_or(&Type::String(String::from("#BB#")))
            .into();

        id.push_str(&time);
        id.push_str(&call.to_uppercase());
        id.push_str(&band.to_uppercase());
        id
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Record {
    /// Compares QSO records by QSO ID which is generated from date, time, call and band
    fn eq(&self, other: &Self) -> bool {
        self.qso_id().eq(&other.qso_id())
    }
}

/// Creates an ADIF QSO record from a vector of tuples.
impl From<Vec<(&str, &str)>> for Record {
    fn from(value: Vec<(&str, &str)>) -> Self {
        let mut rec = Record::new();
        value.iter().for_each(|(k, v)| {
            let _ = rec.insert(k.to_uppercase(), Type::collate(k, v));
        });
        rec
    }
}

impl SerializeADI for Record {
    /// Serializes a `Record` to ADI string
    fn serialize_adi(&self) -> String {
        let mut chunks = self
            .0
            .iter()
            .map(|(k, v)| {
                let v = v.serialize_adi();
                format!("<{}:{}>{}", k, v.len(), v)
            })
            .collect::<Vec<String>>()
            .chunks(WRAP_AT)
            .map(|f| f.join(" "))
            .collect::<Vec<String>>();
        chunks.push("<EOR>".to_string());
        chunks.join("\n")
    }
}

impl DeserializeADI for Record {
    /// Deserializes a `Record` from an ADI string
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), String> {
        let mut start = 0;
        let mut end = 0;
        let mut length = 0;

        while start < adi.as_ref().len() {
            let off = if start == 0 { 0 } else { 1 };
            start = match adi.as_ref()[(end + off + length)..adi.as_ref().len()].find('<') {
                Some(pos) => pos + end + off + length,
                None => break,
            };

            end = match adi.as_ref()[start..adi.as_ref().len()].find('>') {
                Some(pos) => pos + start,
                None => break,
            };

            let tag: &str = &adi.as_ref()[start + 1..end];

            let mut tag_def = tag.split(':');
            match tag_def.next() {
                Some(param) => match tag_def.next() {
                    Some(val_len) => {
                        length = val_len.parse().unwrap_or(0);
                        let val = String::from(&adi.as_ref()[end + 1..end + 1 + length]);
                        self.0.insert(
                            param.to_uppercase(),
                            Type::collate(param.to_uppercase(), val),
                        );
                    }
                    None => break,
                },
                None => break,
            };
        }

        Ok(())
    }
}

impl From<&Record> for HashMap<String, String> {
    fn from(value: &Record) -> Self {
        value.iter().map(|(k, v)| (k.clone(), v.into())).collect()
    }
}

impl From<HashMap<String, String>> for Record {
    fn from(value: HashMap<String, String>) -> Self {
        let mut rec: Record = Default::default();
        value.iter().for_each(|(k, v)| {
            let _ = rec.insert(k, v.to_string().into());
        });
        rec
    }
}

/// Represents an ADIF header
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "serde_impl", feature = "serde_loose"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Header {
    record: Record,
    comment: String,
}

impl Header {
    /// Creates an empty ADIF header
    pub fn new() -> Self {
        Self {
            record: Record::new(),
            comment: String::new(),
        }
    }

    /// Set the comment
    pub fn set_comment(mut self, comment: impl AsRef<str>) -> Self {
        self.comment = comment.as_ref().to_string();
        self
    }

    pub fn comment(&self) -> &String {
        &self.comment
    }

    /// Set the program ID to show in the header
    pub fn set_program_id(mut self, prog_id: impl AsRef<str>) -> Self {
        self.record.insert("PROGRAMID", prog_id.as_ref().into());
        self
    }

    pub fn program_id(&self) -> String {
        self.record
            .get("PROGRAMID")
            .unwrap_or(&Type::String(String::from("")))
            .into()
    }

    /// Set the program version to show in the header
    pub fn set_program_ver(mut self, prog_ver: impl AsRef<str>) -> Self {
        self.record
            .insert("PROGRAMVERSION", prog_ver.as_ref().into());
        self
    }

    pub fn program_ver(&self) -> String {
        self.record
            .get("PROGRAMVERSION")
            .unwrap_or(&Type::String(String::from("")))
            .into()
    }

    /// Set the ADIF version to show in the header
    pub fn set_adif_ver(mut self, adif_ver: impl AsRef<str>) -> Self {
        self.record.insert("ADIF_VER", adif_ver.as_ref().into());
        self
    }

    pub fn record(&self) -> &Record {
        &self.record
    }

    pub fn record_mut(&mut self) -> &mut Record {
        &mut self.record
    }

    /// Check if a field is in the header
    pub fn contains(&self, field: impl AsRef<str>) -> bool {
        self.record.contains(field)
    }

    /// Get the field content
    pub fn get(&self, field: impl AsRef<str>) -> Option<&Type> {
        self.record.get(field)
    }

    /// Add or change a field in the header
    pub fn insert(&mut self, field: impl AsRef<str>, val: Type) -> Option<Type> {
        self.record.insert(field, val)
    }
    /// Returns a double-ended iterator visiting all field-value pairs in order of insertion.
    pub fn iter(&'_ self) -> linked_hash_map::Iter<'_, String, Type> {
        self.record.iter()
    }

    /// Removes and returns the value corresponding to the field name from the record.
    pub fn remove(&mut self, field: impl AsRef<str>) -> Option<Type> {
        self.record.remove(field.as_ref().to_uppercase())
    }

    /// Returns a double-ended iterator visiting all fields in order of insertion.
    pub fn keys(&self) -> Keys<'_, String, Type> {
        self.record.keys()
    }
}

impl Default for Header {
    /// Builds a default header with
    /// - ADIF_VER: "3.1.6"
    /// - PROGRAMID: "adif_io"
    /// - PROGRAMVERSION: _adif_io version_
    /// - CREATED_TIMESTAMP: _current date/time_
    fn default() -> Self {
        let mut header = Record::new();
        header.insert("ADIF_VER", "3.1.6".into());
        header.insert("PROGRAMID", PKG_NAME.into());
        header.insert("PROGRAMVERSION", PKG_VERSION.into());

        let dt = chrono::offset::Utc::now();
        header.insert(
            "CREATED_TIMESTAMP",
            dt.format("%Y%m%d %H%M%S").to_string().into(),
        );

        Self {
            record: header,
            comment: String::from("Generated with adif_io rust crate"),
        }
    }
}

/// Creates an ADIF header record from a vector of tuples
impl From<Vec<(&str, &str)>> for Header {
    fn from(value: Vec<(&str, &str)>) -> Self {
        let mut header = Header::default();
        value.iter().for_each(|(k, v)| {
            let _ = header.insert(k.to_uppercase(), Type::collate(k, v));
        });
        header
    }
}

impl SerializeADI for Header {
    fn serialize_adi(&self) -> String {
        let mut header = self
            .record
            .0
            .iter()
            .map(|(k, v)| {
                let v = v.serialize_adi();
                format!("<{}:{}>{}", k, v.len(), v)
            })
            .collect::<Vec<String>>();
        header.insert(0, self.comment.clone());
        header.push("<EOH>".to_string());

        header.join("\n")
    }
}

impl DeserializeADI for Header {
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), String> {
        let start = adi.as_ref().find('<').unwrap_or(0);
        self.comment = adi.as_ref()[0..start].trim().to_string();
        self.record
            .deserialize_adi(adi.as_ref()[start..adi.as_ref().len()].trim())
    }
}

/// Represents an ADIF document with header and QSO records
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "serde_impl", feature = "serde_loose"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Doc {
    header: Header,
    records: Vec<Record>,
}

impl Doc {
    /// Create an empty ADIF document
    pub fn new() -> Self {
        Self {
            header: Header::default(),
            records: Vec::new(),
        }
    }

    /// Create an ADIF document with header data
    pub fn new_header(
        prog_id: impl AsRef<str>,
        prog_ver: impl AsRef<str>,
        comment: impl AsRef<str>,
    ) -> Self {
        let header = Header::default()
            .set_program_id(prog_id)
            .set_program_ver(prog_ver)
            .set_comment(comment);

        Self {
            header,
            records: Vec::new(),
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn set_header(&mut self, header: Header) {
        self.header = header
    }

    /// Add a Record to the document
    pub fn add_record(&mut self, rec: Record) {
        self.records.push(rec);
    }

    /// Insert a Record at index into the document
    pub fn insert_record(&mut self, index: usize, rec: Record) {
        self.records.insert(index, rec);
    }

    /// Get a Record from the document by index
    pub fn get_record(&self, index: usize) -> Option<&Record> {
        self.records.get(index)
    }

    /// Get a mutable Record from the document by index
    pub fn get_record_mut(&mut self, index: usize) -> Option<&mut Record> {
        self.records.get_mut(index)
    }

    /// Remove a Record from document by index
    pub fn remove_record(&mut self, index: usize) -> Record {
        self.records.remove(index)
    }

    /// Iterate over all Records in the document
    pub fn iter_record(&self) -> Iter<'_, Record> {
        self.records.iter()
    }
}

impl Default for Doc {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializeADI for Doc {
    fn serialize_adi(&self) -> String {
        let mut doc = self
            .records
            .iter()
            .map(|r| r.serialize_adi())
            .collect::<Vec<String>>();
        if !self.header().comment.is_empty() && self.header.record.iter().len() > 0 {
            doc.insert(0, self.header.serialize_adi());
        }
        doc.join("\n\n")
    }
}

impl DeserializeADI for Doc {
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), String> {
        let mut rec_str: &str = adi.as_ref();

        if !adi.as_ref().starts_with('<') {
            let mut head_rec = Regex::split(&RE_HEADER, adi.as_ref());
            let head_str = match head_rec.next() {
                Some(arg) => arg,
                None => return Err("No header but doc does not start with tag".to_string()),
            };

            self.header.deserialize_adi(head_str)?;
            rec_str = head_rec.next().unwrap_or("");
        }

        for rec in Regex::split(&RE_RECORD, rec_str) {
            if rec.trim().is_empty() {
                continue;
            }
            let mut record = Record::new();
            record.deserialize_adi(rec.trim())?;
            self.records.push(record);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_010_header() {
        let mut adi_doc = Doc::new_header("Test", "1.1", "Comment");
        adi_doc.header_mut().remove("CREATED_TIMESTAMP");
        let adi_str = adi_doc.serialize_adi();

        assert_eq!(
            adi_str,
            "Comment\n<ADIF_VER:5>3.1.6\n<PROGRAMID:4>Test\n<PROGRAMVERSION:3>1.1\n<EOH>"
        );
    }

    #[test]
    fn test_015_empty_header() {
        let mut adi_doc = Doc::new();
        adi_doc.header = Header::new();
        let mut adi_rec = Record::new();
        adi_rec.insert("QSO_DATE", "20231008".into());
        adi_rec.insert("TIME_ON", "1145".into());
        adi_rec.insert("Call", "dl4bdf".into());
        adi_rec.insert("name", "Walter".into());

        let gs_p = String::from("GRIDSQUARE");
        let gs_v = String::from("JO30uu");
        adi_rec.insert(&gs_p, gs_v.into());

        adi_doc.add_record(adi_rec);

        let adi_str = adi_doc.serialize_adi();

        assert_eq!(
            adi_str,
            "<QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>dl4bdf <NAME:6>Walter <GRIDSQUARE:6>JO30uu\n<EOR>"
        );
    }

    #[test]
    fn test_020_record() {
        let mut adi_doc = Doc::new_header(&String::from("T"), "1", "C");
        adi_doc.header_mut().remove("CREATED_TIMESTAMP");
        let mut adi_rec = Record::new();
        adi_rec.insert("QSO_DATE".to_string(), Type::Date("20231008".to_string()));
        adi_rec.insert("TIME_ON".to_string(), Type::Time("1145".to_string()));
        adi_rec.insert("Call".to_string(), "dl4bdf".into());
        adi_rec.insert("name".to_string(), "Walter".into());
        adi_doc.add_record(adi_rec);

        let adi_str = adi_doc.serialize_adi();

        assert_eq!(
            adi_str,
            "C\n<ADIF_VER:5>3.1.6\n<PROGRAMID:1>T\n<PROGRAMVERSION:1>1\n<EOH>\n\n<QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>dl4bdf <NAME:6>Walter\n<EOR>"
        );
    }

    #[test]
    fn test_025_records() {
        let mut adi_doc = Doc::new_header("T", "1", "C");
        adi_doc.header_mut().remove("CREATED_TIMESTAMP");
        let mut adi_rec1 = Record::new();
        adi_rec1.insert("QSO_DATE", "20231008".into());
        adi_rec1.insert("TIME_ON", "1145".into());
        adi_rec1.insert("Call", "dl4bdf".into());
        adi_rec1.insert("name", "Walter".into());
        adi_doc.add_record(adi_rec1);

        adi_doc.add_record(Record::from(vec![
            ("QSO_DATE", "20231009"),
            ("TIME_ON", "1245"),
            ("Call", "DK5XXX"),
            ("name", "Chris"),
        ]));

        let adi_str = adi_doc.serialize_adi();

        assert_eq!(
            adi_str,
            "C\n<ADIF_VER:5>3.1.6\n<PROGRAMID:1>T\n<PROGRAMVERSION:1>1\n<EOH>\n\n<QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>dl4bdf <NAME:6>Walter\n<EOR>\n\n<QSO_DATE:8>20231009 <TIME_ON:4>1245 <CALL:6>DK5XXX <NAME:5>Chris\n<EOR>"
        );
    }

    #[test]
    fn test_030_records_ln() {
        let mut adi_doc = Doc::new_header("T", "1", "C");
        adi_doc.header_mut().remove("CREATED_TIMESTAMP");
        let mut adi_rec1 = Record::new();
        adi_rec1.insert("QSO_DATE", "20231008".into());
        adi_rec1.insert("TIME_ON", "1145".into());
        adi_rec1.insert("Call", "dl4bdf".into());
        adi_rec1.insert("name", "Walter".into());
        adi_rec1.insert("my_name", "Andy".into());
        adi_rec1.insert("STATION_CALLSIGN", "DF1ASC".into());
        adi_doc.add_record(adi_rec1);

        adi_doc.add_record(Record::from(vec![
            ("QSO_DATE", "20231009"),
            ("TIME_ON", "1245"),
            ("Call", "DK5XXX"),
            ("name", "Chris"),
        ]));

        let adi_str = adi_doc.serialize_adi();

        assert_eq!(
            adi_str,
            "C\n<ADIF_VER:5>3.1.6\n<PROGRAMID:1>T\n<PROGRAMVERSION:1>1\n<EOH>\n\n<QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>dl4bdf <NAME:6>Walter <MY_NAME:4>Andy\n<STATION_CALLSIGN:6>DF1ASC\n<EOR>\n\n<QSO_DATE:8>20231009 <TIME_ON:4>1245 <CALL:6>DK5XXX <NAME:5>Chris\n<EOR>"
        );
    }

    #[test]
    fn test_050_records_eq() {
        let mut adi_rec1 = Record::new();
        adi_rec1.insert("QSO_DATE", "20231008".into());
        adi_rec1.insert("TIME_ON", "1145".into());
        adi_rec1.insert("call", "dl4bdf".into());
        adi_rec1.insert("band", "40m".into());
        adi_rec1.insert("name", "Walter".into());

        let mut adi_rec2 = Record::new(); // Created different but the same
        adi_rec2.insert("QSO_DATE", "20231008".into());
        adi_rec2.insert("tIME_ON", "1145".into());
        adi_rec2.insert("Call", "Dl4bdf".into());
        adi_rec2.insert("banD", "40M".into());
        adi_rec2.insert("Name", "Karl".into()); // Different name

        let mut adi_rec3 = adi_rec1.clone();
        adi_rec3.insert("qSO_DATE", "20231009".into());

        let mut adi_rec4 = adi_rec1.clone(); // Missing fields
        adi_rec4.remove("CALL");
        adi_rec4.remove("band");

        let adi_rec5 = Record::new();
        let adi_rec6 = Record::new();

        assert_eq!(true, adi_rec1 == adi_rec2);
        assert_eq!(false, adi_rec1 == adi_rec3);
        assert_eq!(false, adi_rec1 == adi_rec4);
        assert_eq!(true, adi_rec5 == adi_rec6);
    }

    #[test]
    fn test_100_adi_header() {
        let adi_str =
            "<ADIF_VER:5>3.1.4\n<PROGRAMID:4>Test\n<PROGRAMVERSION:3>3.2<USERDEF1:3:N>XXX";

        let mut header = Header::new();
        header.deserialize_adi(adi_str).unwrap();
        assert_eq!(true, header.contains("ADIF_VER"));
        assert_eq!(true, header.contains("PROGRAMID"));
    }

    #[test]
    fn test_110_adi_record() {
        let adi_str = "<qso_DATE:8>20231008 <TIME_on:4>1145 <CALL:6>dl4bdf <NAME:6>Walter <DISTANCE:5>123.4 <QSO_RANDOM:1>y <SWL:1>n <GRIDSQUARE:4>jo30 <MY_GRIDSQUARE:5>jo30x";

        let mut rec = Record::new();
        rec.deserialize_adi(adi_str).unwrap();
        // assert!(rec.contains("QSO_dATE"));
        // assert!(rec.contains("tIME_ON"));
        assert!(rec.contains("CaLL"));
        assert!(rec.contains("NAMe"));

        assert_eq!(
            Some(&Type::Date("2023-10-08".to_string())),
            rec.get("QSO_DATE")
        );
        assert_eq!(Some(&Type::Time("11:45".to_string())), rec.get("TIME_on"));
        assert_eq!(Some(&Type::Number(123.4)), rec.get("DISTANCE"));
        assert_eq!(Some(&Type::Boolean(true)), rec.get("qso_RANDOM"));
        assert_eq!(Some(&Type::Boolean(false)), rec.get("SWL"));
        assert_eq!(
            Some(&Type::GridSquare("jo30".to_string())),
            rec.get("GRIDSQUARE")
        );
        assert_eq!(
            Some(&Type::String("jo30x".to_string())),
            rec.get("MY_GRIDSQUARE")
        );
    }

    #[test]
    fn test_120_adidoc() {
        let adi_str = "C\n<ADIF_VER:5>3.1.4\n<PROGRAMID:4>Test\n<PROGRAMVERSION:3>3.2<USERDEF1:3:N>XXX\n<eoh>\n\n<qso_DATE:8>20231008 <TIME_on:4>1145 <CALL:6>dl4bdf <NAME:6>Walter\n<eor>\n\n<QSO_DATE:8>20231009 <TIME_ON:4>1245 <CALL:6>DK7DCM <NAME:5>Chris\n<eor>\n\n";

        let mut doc = Doc::new();
        doc.deserialize_adi(adi_str).unwrap();
        assert_eq!("C", doc.header.comment);
        assert!(doc.header.contains("ADIF_VER"));
        assert!(doc.header.contains("PROGRAMID"));

        assert_eq!(2, doc.records.len());
        for rec in doc.records.iter() {
            assert!(rec.contains("QSO_DATE"));
            assert!(rec.contains("CALL"));
        }
    }
}

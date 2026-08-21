//! Library to read and write ADIF formated files
//
// licensed under CC BY-SA 4.0,
// To view a copy of this license, visit <http://creativecommons.org/licenses/by-sa/4.0/>
//
// Author: Andreas, <df1asc@darc.de>
//!
//! # Example
//!
// This example does not print anything if run as a doctest
//! ```
//! use std::fs;
//! use adif_io::{DeserializeADI, SerializeADI, Doc, Record};
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
//!     println!("QSO count: {}", doc.iter_records().count());
//!     doc.iter_records().enumerate().for_each(|(i, qso)| println!("QSO {}: {}", i+1, qso));
//!
//!     // Get 6th QSO and modify data
//!     let qso = doc.get_record_mut(5).expect("no QSO available");
//!     qso.insert("NOTES", "New data".into()); // NOTES field
//!     qso["CAll"] = "AB3ABC".into(); // Change callsign
//!
//!     // Create a `Record` and add it, case for field names does not matter
//!     let qso = Record::from(vec![
//!         ("QSO_DATE", "20231009"),
//!         ("TIME_ON", "1245"),
//!         ("Call", "DK5XXX"),  // Mixed case filed name inserted
//!         ("NAME", "Chris"),  // Upper case field name inserted
//!     ]);
//!     assert_eq!("Chris", qso["name"].to_string());  // Accessed field name with lower case
//!     assert_eq!("DK5XXX", qso["caLL"].to_string());  // Accessed field name with mixed case
//!     doc.add_record(qso);
//!
//!     // Serialize and write
//!     fs::write("example.adi", doc.serialize_adi()).expect("could not write ADI output file");
//! }
//! ```

#[cfg(all(feature = "serde_impl", feature = "serde_loose"))]
compile_error!("feature 'serde_impl' and 'serde_loose' cannot be enabled at the same time");

mod gridsquare;
pub use gridsquare::{Coordinate, GridSquare, Vector};

mod error;
pub use error::Error;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use linked_hash_map::{Keys, LinkedHashMap};
use regex::Regex;
#[cfg(any(feature = "serde_impl", feature = "serde_loose"))]
use serde;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};
use std::str::FromStr;
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
    /// Serialize ADIF data to ADIF formated String.
    fn serialize_adi(&self) -> String;

    /// Builds an ADI tag from a key/value pair.
    fn build_tag(key: &String, value: &Type) -> String {
        let val = value.serialize_adi();
        if key.starts_with("APP_")
            && let Some(td) = value.to_type_def()
        {
            format!("<{}:{}:{}>{}", key, val.len(), td, val)
        } else {
            format!("<{}:{}>{}", key, val.len(), val)
        }
    }
}

/// Converting QSO data from an ADI `String`
pub trait DeserializeADI {
    /// Deserialize ADIF data from ADIF formated String
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), Error>;
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
    Integer(i64),
    PositiveInteger(u64),
    Number(f64),
    Boolean(bool),
    Date(NaiveDate),
    Time(NaiveTime),
    GridSquare(GridSquare),
    UserDef {
        field_type: String,
        definition: String,
    },
}

impl Type {
    /// Collate the value type from the field name if possible or fall back to `String`.
    pub fn collate(field: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let value = value.as_ref();
        if let Some(t) = TYPE_MAP.get(&field.as_ref().to_uppercase()) {
            match t.as_str() {
                "Integer" => Self::build_integer(value),
                "PositiveInteger" => Self::build_pos_integer(value),
                "Number" => Self::build_number(value),
                "Boolean" => Self::build_boolean(value),
                "Date" => Self::build_date(value),
                "Time" => Self::build_time(value),
                "GridSquare" => Self::build_grid(value),
                _ => Type::String(value.to_string()),
            }
        } else {
            Type::String(value.to_string())
        }
    }

    /// Build a `PositiveInteger` or fallback to `String`.
    fn build_pos_integer(value: &str) -> Type {
        if let Ok(v) = value.parse::<u64>() {
            Type::PositiveInteger(v)
        } else {
            Type::String(value.to_string())
        }
    }

    /// Build a `Integer` or fallback to `String`.
    fn build_integer(value: &str) -> Type {
        if let Ok(v) = value.parse::<i64>() {
            Type::Integer(v)
        } else {
            Type::String(value.to_string())
        }
    }

    /// Build a `Boolean` or fallback to `String`.
    fn build_boolean(value: &str) -> Type {
        if value.to_uppercase() == "Y" {
            Type::Boolean(true)
        } else {
            Type::Boolean(false)
        }
    }

    /// Build a `Number` or fallback to `String`.
    fn build_number(value: &str) -> Type {
        if let Ok(v) = value.parse::<f64>() {
            Type::Number(v)
        } else {
            Type::String(value.to_string())
        }
    }

    /// Build a `Time` or fallback to `String`.
    fn build_time(value: &str) -> Type {
        if (value.len() == 4 || value.len() == 6)
            && str_is_digits(value)
            && let Ok(time) = NaiveTime::from_str(
                value
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(2)
                    .map(|c| c.iter().collect())
                    .collect::<Vec<String>>()
                    .join(":")
                    .as_str(),
            )
        {
            Type::Time(time)
        } else {
            Type::String(value.to_string())
        }
    }

    /// Build a `Date` or fallback to `String`.
    fn build_date(value: &str) -> Type {
        if value.len() == 8
            && str_is_digits(value)
            && let Ok(date) = NaiveDate::from_str(
                format!("{}-{}-{}", &value[..4], &value[4..6], &value[6..8]).as_str(),
            )
        {
            Type::Date(date)
        } else {
            Type::String(value.to_string())
        }
    }

    /// Build a `GridSquare` or fallback to `String`.
    fn build_grid(value: &str) -> Type {
        if let Ok(grid) = GridSquare::new(value) {
            Type::GridSquare(grid)
        } else {
            Type::String(value.to_string())
        }
    }

    /// Derive type from type definition in APP_x and USERDEFx fields.
    pub fn from_type_def(def: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let value = value.as_ref();
        match def.as_ref() {
            "N" => Self::build_number(value),
            "B" => Self::build_boolean(value),
            "D" => Self::build_date(value),
            "T" => Self::build_time(value),
            _ => Type::String(value.to_string()),
        }
    }

    /// Get type definition from `Type`.
    pub fn to_type_def(&self) -> Option<&'static str> {
        match self {
            Type::String(_) => Some("S"),
            Type::Number(_) => Some("N"),
            Type::Boolean(_) => Some("B"),
            Type::Date(_) => Some("D"),
            Type::Time(_) => Some("T"),
            _ => None,
        }
    }
}

impl From<&Type> for String {
    fn from(value: &Type) -> Self {
        match value {
            Type::String(v) => v.clone(),
            Type::Integer(v) => v.to_string(),
            Type::PositiveInteger(v) => v.to_string(),
            Type::Number(v) => v.to_string(),
            Type::Boolean(v) => v.to_string(),
            Type::Date(v) => v.to_string(),
            Type::Time(v) => v.to_string(),
            Type::GridSquare(v) => v.to_string(),
            Type::UserDef {
                field_type,
                definition,
            } => format!("{}, type {}", definition, field_type),
        }
    }
}

impl Display for Type {
    /// Formats the value without typing info
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&Type as Into<String>>::into(self))
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

impl From<f64> for Type {
    fn from(value: f64) -> Self {
        Type::Number(value)
    }
}

impl From<f32> for Type {
    fn from(value: f32) -> Self {
        Type::Number(value.into())
    }
}

impl From<i64> for Type {
    fn from(value: i64) -> Self {
        Type::Integer(value.into())
    }
}

impl From<i32> for Type {
    fn from(value: i32) -> Self {
        Type::Integer(value.into())
    }
}

impl From<i16> for Type {
    fn from(value: i16) -> Self {
        Type::Integer(value.into())
    }
}

impl From<i8> for Type {
    fn from(value: i8) -> Self {
        Type::Integer(value.into())
    }
}

impl From<u64> for Type {
    fn from(value: u64) -> Self {
        Type::PositiveInteger(value.into())
    }
}

impl From<u32> for Type {
    fn from(value: u32) -> Self {
        Type::PositiveInteger(value.into())
    }
}

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        Type::PositiveInteger(value.into())
    }
}

impl From<u8> for Type {
    fn from(value: u8) -> Self {
        Type::PositiveInteger(value.into())
    }
}

impl From<NaiveDate> for Type {
    fn from(value: NaiveDate) -> Self {
        Type::Date(value)
    }
}

impl From<NaiveTime> for Type {
    fn from(value: NaiveTime) -> Self {
        Type::Time(value)
    }
}

impl From<GridSquare> for Type {
    fn from(value: GridSquare) -> Self {
        Type::GridSquare(value)
    }
}

impl SerializeADI for Type {
    fn serialize_adi(&self) -> String {
        match self {
            Type::String(v) => v.to_string(),
            Type::Integer(v) => v.to_string(),
            Type::PositiveInteger(v) => v.to_string(),
            Type::Number(v) => v.to_string(),
            Type::Boolean(v) => {
                if *v {
                    "Y".to_string()
                } else {
                    "N".to_string()
                }
            }
            Type::Date(v) => v.to_string().replace("-", ""),
            Type::Time(v) => v.to_string().replace(":", ""),
            Type::GridSquare(v) => v.to_string(),
            Type::UserDef {
                field_type: _,
                definition,
            } => definition.to_string(),
        }
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
            Type::Integer(v) => serializer.serialize_i64(*v),
            Type::PositiveInteger(v) => serializer.serialize_u64(*v),
            Type::Date(v) => serializer.serialize_str(&v.format("%Y%m%d").to_string()),
            Type::Time(v) => serializer.serialize_str(&v.format("%H%M%S").to_string()),
            Type::UserDef {
                field_type: _,
                definition,
            } => serializer.serialize_str(definition),
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
impl serde::de::Visitor<'_> for TypeVisitor {
    type Value = Type;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("ADIF compatible data")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Type::Boolean(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Type::Integer(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Type::PositiveInteger(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Type::Number(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
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

    /// Add or change a field to a QSO record and collate `Type`
    pub fn insert_str(&mut self, field: impl AsRef<str>, val: impl AsRef<str>) -> Option<Type> {
        self.0
            .insert(field.as_ref().to_uppercase(), Type::collate(field, val))
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

    /// Get a `DateTime` from QSO start if available
    pub fn date_time_on(&self) -> Option<DateTime<Utc>> {
        if let (Some(Type::Date(d_on)), Some(Type::Time(t_on))) =
            (self.get("QSO_DATE"), self.get("TIME_ON"))
        {
            let dt = NaiveDateTime::new(*d_on, *t_on);
            Some(dt.and_utc())
        } else {
            None
        }
    }

    /// Get a `DateTime` from QSO end if available
    pub fn date_time_off(&self) -> Option<DateTime<Utc>> {
        if let (Some(Type::Date(d_on)), Some(Type::Time(t_on))) = (
            self.get("QSO_DATE_OFF").or_else(|| self.get("QSO_DATE")),
            self.get("TIME_OFF"),
        ) {
            let dt = NaiveDateTime::new(*d_on, *t_on);
            Some(dt.and_utc())
        } else {
            None
        }
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

impl Display for Record {
    /// Formats the record values without typing info
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut display: Vec<String> = Vec::new();
        for (k, v) in self.iter() {
            display.push(format!(
                "\"{k}\": {}",
                match v {
                    Type::String(v) => format!("\"{v}\""),
                    Type::Integer(v) => format!("{v}"),
                    Type::PositiveInteger(v) => format!("{v}"),
                    Type::Number(v) => format!("{v}"),
                    Type::Boolean(v) => format!("{v}"),
                    Type::Date(v) => format!("\"{v}\""),
                    Type::Time(v) => format!("\"{v}\""),
                    Type::GridSquare(v) => format!("\"{v}\""),
                    Type::UserDef {
                        field_type,
                        definition,
                    } => format!("<{}, type {}>", definition, field_type),
                }
            ));
        }
        write!(f, "Record({{{}}})", display.join(", "))
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
            .map(|(k, v)| Self::build_tag(k, v))
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
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), Error> {
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

                        if param.to_uppercase().starts_with("USERDEF") {
                            let field_type = if let Some(ft) = tag_def.next() {
                                ft.to_uppercase()
                            } else {
                                "".to_string()
                            };

                            self.insert(
                                param,
                                Type::UserDef {
                                    field_type,
                                    definition: val,
                                },
                            )
                        } else if param.to_uppercase().starts_with("APP_") {
                            let field_type = if let Some(ft) = tag_def.next() {
                                ft.to_uppercase()
                            } else {
                                "".to_string()
                            };

                            self.insert(param, Type::from_type_def(field_type, val))
                        } else {
                            self.insert_str(param, val)
                        }
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

impl<'a, T: ?Sized> Index<&'a T> for Record
where
    T: AsRef<str>,
{
    type Output = Type;

    fn index(&self, index: &'a T) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<'a, T: ?Sized> IndexMut<&'a T> for Record
where
    T: AsRef<str>,
{
    fn index_mut(&mut self, index: &'a T) -> &mut Type {
        self.get_mut(index).expect("no entry found for key")
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

    /// Check if a field is in the header
    pub fn contains(&self, field: impl AsRef<str>) -> bool {
        self.record.contains(field)
    }

    /// Get the field content
    pub fn get(&self, field: impl AsRef<str>) -> Option<&Type> {
        self.record.get(field)
    }

    /// Get the mutable field content.
    pub fn get_mut(&mut self, field: impl AsRef<str>) -> Option<&mut Type> {
        self.record.get_mut(&field.as_ref().to_uppercase())
    }

    /// Add or change a field in the header
    pub fn insert(&mut self, field: impl AsRef<str>, val: Type) -> Option<Type> {
        self.record.insert(field, val)
    }

    /// Add or change a field in the header and collate `Type`
    pub fn insert_str(&mut self, field: impl AsRef<str>, val: impl AsRef<str>) -> Option<Type> {
        self.record.insert_str(field, val)
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
        header.insert_str("ADIF_VER", "3.1.6");
        header.insert_str("PROGRAMID", PKG_NAME);
        header.insert_str("PROGRAMVERSION", PKG_VERSION);

        header.insert_str(
            "CREATED_TIMESTAMP",
            Utc::now().format("%Y%m%d %H%M%S").to_string(),
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
            let _ = header.insert_str(k, v);
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
                if let Type::UserDef {
                    field_type,
                    definition,
                } = v
                {
                    if !field_type.is_empty() {
                        format!("<{}:{}:{}>{}", k, definition.len(), field_type, definition)
                    } else {
                        format!("<{}:{}>{}", k, definition.len(), definition)
                    }
                } else {
                    Self::build_tag(k, v)
                }
            })
            .collect::<Vec<String>>();
        header.insert(0, self.comment.clone());
        header.push("<EOH>".to_string());

        header.join("\n")
    }
}

impl DeserializeADI for Header {
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), Error> {
        let start = adi.as_ref().find('<').unwrap_or(0);
        self.comment = adi.as_ref()[0..start].trim().to_string();
        self.record
            .deserialize_adi(adi.as_ref()[start..adi.as_ref().len()].trim())
    }
}

impl<'a, T: ?Sized> Index<&'a T> for Header
where
    T: AsRef<str>,
{
    type Output = Type;

    fn index(&self, index: &'a T) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<'a, T: ?Sized> IndexMut<&'a T> for Header
where
    T: AsRef<str>,
{
    fn index_mut(&mut self, index: &'a T) -> &mut Type {
        self.get_mut(index).expect("no entry found for key")
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
    pub fn iter_records(&self) -> Iter<'_, Record> {
        self.records.iter()
    }

    /// Iterate over all Records in the document and allow modifying
    pub fn iter_records_mut(&mut self) -> IterMut<'_, Record> {
        self.records.iter_mut()
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
    fn deserialize_adi(&mut self, adi: impl AsRef<str>) -> Result<(), Error> {
        let mut rec_str: &str = adi.as_ref();

        if !adi.as_ref().starts_with('<') {
            let mut head_rec = Regex::split(&RE_HEADER, adi.as_ref());
            let head_str = match head_rec.next() {
                Some(arg) => arg,
                None => return Err(Error::DeserializeMissingHeader),
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
        let mut adi_rec = Record::default();
        adi_rec.insert_str("QSO_DATE", "20231008");
        adi_rec.insert_str("TIME_ON", "1145");
        adi_rec.insert("Call".to_string(), "dl4bdf".into());
        adi_rec.insert("name".to_string(), "Walter".into());
        adi_doc.add_record(adi_rec);

        let adi_str = adi_doc.serialize_adi();

        assert_eq!(
            adi_str,
            "C\n<ADIF_VER:5>3.1.6\n<PROGRAMID:1>T\n<PROGRAMVERSION:1>1\n<EOH>\n\n\
            <QSO_DATE:8>20231008 <TIME_ON:6>114500 <CALL:6>dl4bdf <NAME:6>Walter\n<EOR>"
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
            "C\n<ADIF_VER:5>3.1.6\n<PROGRAMID:1>T\n<PROGRAMVERSION:1>1\n<EOH>\n\n\
            <QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>dl4bdf <NAME:6>Walter\n<EOR>\n\n\
            <QSO_DATE:8>20231009 <TIME_ON:6>124500 <CALL:6>DK5XXX <NAME:5>Chris\n<EOR>"
        );
    }

    #[test]
    fn test_030_records_ln() {
        let mut adi_doc = Doc::new_header("T", "1", "C");
        adi_doc.header_mut().remove("CREATED_TIMESTAMP");
        let mut adi_rec1 = Record::new();
        adi_rec1.insert_str("QSO_DATE", "20231008");
        adi_rec1.insert_str("TIME_ON", "1145");
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
            "C\n<ADIF_VER:5>3.1.6\n<PROGRAMID:1>T\n<PROGRAMVERSION:1>1\n<EOH>\n\n\
            <QSO_DATE:8>20231008 <TIME_ON:6>114500 <CALL:6>dl4bdf <NAME:6>Walter <MY_NAME:4>Andy\n<STATION_CALLSIGN:6>DF1ASC\n<EOR>\n\n\
            <QSO_DATE:8>20231009 <TIME_ON:6>124500 <CALL:6>DK5XXX <NAME:5>Chris\n<EOR>"
        );
    }

    #[test]
    fn test_050_records_eq() {
        let mut adi_rec1 = Record::new();
        adi_rec1.insert_str("QSO_DATE", "20231008");
        adi_rec1.insert_str("TIME_ON", "1145");
        adi_rec1.insert("call", "dl4bdf".into());
        adi_rec1.insert("band", "40m".into());
        adi_rec1.insert("name", "Walter".into());

        let mut adi_rec2 = Record::new(); // Created different but the same
        adi_rec2.insert_str("QSO_DATE", "20231008");
        adi_rec2.insert_str("tIME_ON", "1145");
        adi_rec2.insert("Call", "Dl4bdf".into());
        adi_rec2.insert("banD", "40M".into());
        adi_rec2.insert("Name", "Karl".into()); // Different name

        let mut adi_rec3 = adi_rec1.clone();
        adi_rec3.insert_str("qSO_DATE", "20231009");

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

        let mut header = Header::default();
        header.deserialize_adi(adi_str).unwrap();
        assert_eq!(true, header.contains("ADIF_VER"));
        assert_eq!(true, header.contains("PROGRAMID"));
    }

    #[test]
    fn test_105_adi_header_info() {
        let adi_str =
            "Comment<ADIF_VER:5>3.1.4\n<PROGRAMID:4>Test\n<PROGRAMVERSION:3>3.2<USERDEF1:3:N>XXX";

        let mut header = Header::new();
        header.deserialize_adi(adi_str).unwrap();
        assert_eq!(header.comment(), "Comment");
        assert_eq!(header.program_id(), "Test");
        assert_eq!(header.program_ver(), "3.2");
    }

    #[test]
    fn test_110_adi_record() {
        let adi_str = "<qso_DATE:8>20231008 <TIME_on:4>1145 <CALL:6>dl4bdf <NAME:6>Walter \
        <DISTANCE:5>123.4 <QSO_RANDOM:1>y <SWL:1>n <GRIDSQUARE:4>jo30 <MY_GRIDSQUARE:5>jo30x";

        let mut rec = Record::new();
        rec.deserialize_adi(adi_str).unwrap();
        // assert!(rec.contains("QSO_dATE"));
        // assert!(rec.contains("tIME_ON"));
        assert!(rec.contains("CaLL"));
        assert!(rec.contains("NAMe"));

        assert_eq!(
            Some(&Type::collate("QSO_DATE", "20231008")),
            rec.get("QSO_DATE")
        );
        assert_eq!(Some(&Type::collate("TIME_ON", "1145")), rec.get("TIME_on"));
        assert_eq!(Some(&Type::Number(123.4)), rec.get("DISTANCE"));
        assert_eq!(Some(&Type::Boolean(true)), rec.get("qso_RANDOM"));
        assert_eq!(Some(&Type::Boolean(false)), rec.get("SWL"));
        assert_eq!(
            Some(&Type::GridSquare(GridSquare::new("jo30").unwrap())),
            rec.get("GRIDSQUARE")
        );
        assert_eq!(
            Some(&Type::String("jo30x".to_string())),
            rec.get("MY_GRIDSQUARE")
        );
    }

    #[test]
    fn test_115_record_disp() {
        let adi_str = "<qso_DATE:8>20231008 <TIME_on:4>1145 <CALL:6>dl4bdf <NAME:6>Walter \
        <DISTANCE:5>123.4 <QSO_RANDOM:1>y <SWL:1>n <GRIDSQUARE:4>jo30 <MY_GRIDSQUARE:5>jo30x \
        <NR_BURSTS:1>9 <CQZ:3>123";

        let mut rec = Record::new();
        rec.deserialize_adi(adi_str).unwrap();

        assert_eq!(
            rec.to_string(),
            "\
        Record({\"QSO_DATE\": \"2023-10-08\", \"TIME_ON\": \"11:45:00\", \"CALL\": \"dl4bdf\", \
        \"NAME\": \"Walter\", \"DISTANCE\": 123.4, \"QSO_RANDOM\": true, \"SWL\": false, \
        \"GRIDSQUARE\": \"JO30\", \"MY_GRIDSQUARE\": \"jo30x\", \"NR_BURSTS\": 9, \"CQZ\": 123})"
        );
    }

    #[test]
    fn test_120_adidoc() {
        let adi_str = "\
        C\n<ADIF_VER:5>3.1.4\n<PROGRAMID:4>Test\n<PROGRAMVERSION:3>3.2<USERDEF1:3:N>XXX\n<eoh>\n\n\
        <qso_DATE:8>20231008 <TIME_on:4>1145 <CALL:6>dl4bdf <NAME:6>Walter\n<eor>\n\n\
        <QSO_DATE:8>20231009 <TIME_ON:4>1245 <CALL:6>DK7DCM <NAME:5>Chris\n<eor>\n\n";

        let mut doc = Doc::default();
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

    #[test]
    fn test_200_type_collation() {
        assert_eq!(
            Type::String("Test".to_string()),
            Type::collate("NAME", "Test")
        );

        assert_eq!(Type::Integer(-123), Type::collate("NR_BURSTS", "-123"));
        assert_eq!(Type::Integer(123), Type::collate("NR_BURSTS", "123"));
        assert_eq!(
            Type::String("abc".to_string()),
            Type::collate("NR_BURSTS", "abc")
        );

        assert_eq!(Type::PositiveInteger(123), Type::collate("CQZ", "123"));
        assert_eq!(
            Type::String("-123".to_string()),
            Type::collate("CQZ", "-123")
        );

        assert_eq!(Type::Number(123.4), Type::collate("DISTANCE", "123.4"));
        assert_eq!(
            Type::String("k123.4".to_string()),
            Type::collate("DISTANCE", "k123.4")
        );

        assert_eq!(Type::Boolean(true), Type::collate("QSO_RANDOM", "Y"));
        assert_eq!(Type::Boolean(false), Type::collate("SWL", "N"));
        assert_eq!(Type::Boolean(false), Type::collate("SWL", ""));
        assert_eq!(Type::Boolean(false), Type::collate("SWL", "*"));

        assert_eq!(
            Type::Date(NaiveDate::from_str("2026-03-04").unwrap()),
            Type::collate("QSO_DATE", "20260304")
        );
        assert_eq!(
            Type::String("2026030".to_string()),
            Type::collate("QSO_DATE", "2026030")
        );
        assert_eq!(
            Type::String("2026030k".to_string()),
            Type::collate("QSO_DATE", "2026030k")
        );

        assert_eq!(
            Type::Time(NaiveTime::from_str("11:45").unwrap()),
            Type::collate("TIME_ON", "1145")
        );
        assert_eq!(
            Type::String("114".to_string()),
            Type::collate("TIME_ON", "114")
        );
        assert_eq!(
            Type::String("114k".to_string()),
            Type::collate("TIME_ON", "114k")
        );

        assert_eq!(
            Type::GridSquare(GridSquare::new("JO30ui").unwrap()),
            Type::collate("GRIDSQUARE", "jo30ui")
        );
        assert_eq!(
            Type::String("JO30u".to_string()),
            Type::collate("GRIDSQUARE", "JO30u")
        );
        assert_eq!(
            Type::String("".to_string()),
            Type::collate("GRIDSQUARE", "")
        );
    }

    #[test]
    fn test_210_type_into_string() {
        // Into
        assert_eq!(
            "Test".to_string(),
            <&Type as Into<String>>::into(&Type::String("Test".to_string()))
        );
        assert_eq!(
            "-123".to_string(),
            <&Type as Into<String>>::into(&Type::Integer(-123))
        );
        assert_eq!(
            "123".to_string(),
            <&Type as Into<String>>::into(&Type::PositiveInteger(123))
        );
        assert_eq!(
            "123.4".to_string(),
            <&Type as Into<String>>::into(&Type::Number(123.4))
        );
        assert_eq!(
            "true".to_string(),
            <&Type as Into<String>>::into(&Type::Boolean(true))
        );
        assert_eq!(
            "false".to_string(),
            <&Type as Into<String>>::into(&Type::Boolean(false))
        );
        assert_eq!(
            "11:45:00".to_string(),
            <&Type as Into<String>>::into(&Type::Time(NaiveTime::from_str("11:45").unwrap()))
        );
        assert_eq!(
            "2026-03-04".to_string(),
            <&Type as Into<String>>::into(&Type::Date(NaiveDate::from_str("2026-03-04").unwrap()))
        );
        assert_eq!(
            "JO30ui".to_string(),
            <&Type as Into<String>>::into(&Type::GridSquare(GridSquare::new("JO30ui").unwrap()))
        );

        // Display
        assert_eq!(
            "Test".to_string(),
            Type::String("Test".to_string()).to_string()
        );
        assert_eq!("-123".to_string(), Type::Integer(-123).to_string());
        assert_eq!("123".to_string(), Type::PositiveInteger(123).to_string());
        assert_eq!("123.4".to_string(), Type::Number(123.4).to_string());

        assert_eq!("true".to_string(), Type::Boolean(true).to_string());
        assert_eq!("false".to_string(), Type::Boolean(false).to_string());

        assert_eq!(
            "11:45:00".to_string(),
            Type::Time(NaiveTime::from_str("11:45").unwrap()).to_string()
        );
        assert_eq!(
            "2026-03-04".to_string(),
            Type::Date(NaiveDate::from_str("2026-03-04").unwrap()).to_string()
        );

        assert_eq!(
            "JO30ui".to_string(),
            Type::GridSquare(GridSquare::new("JO30ui").unwrap()).to_string()
        );
    }

    #[test]
    fn test_220_type_serialize() {
        assert_eq!(
            "Test".to_string(),
            Type::String("Test".to_string()).serialize_adi()
        );

        assert_eq!("-123".to_string(), Type::Integer(-123).serialize_adi());
        assert_eq!("123".to_string(), Type::Integer(123).serialize_adi());

        assert_eq!(
            "123".to_string(),
            Type::PositiveInteger(123).serialize_adi()
        );

        assert_eq!("123.4".to_string(), Type::Number(123.4).serialize_adi());
        assert_eq!("-123.4".to_string(), Type::Number(-123.4).serialize_adi());

        assert_eq!("Y".to_string(), Type::Boolean(true).serialize_adi());
        assert_eq!("N".to_string(), Type::Boolean(false).serialize_adi());

        assert_eq!(
            "114500".to_string(),
            Type::Time(NaiveTime::from_str("11:45").unwrap()).serialize_adi()
        );

        assert_eq!(
            "20260304".to_string(),
            Type::Date(NaiveDate::from_str("2026-03-04").unwrap()).serialize_adi()
        );

        assert_eq!(
            "JO30ui".to_string(),
            Type::GridSquare(GridSquare::new("JO30ui").unwrap()).serialize_adi()
        );
    }

    #[test]
    fn test_230_into_type() {
        assert_eq!(
            Type::String("Test".to_string()),
            <&str as Into<Type>>::into("Test")
        );
        assert_eq!(
            Type::String("Test".to_string()),
            <String as Into<Type>>::into("Test".to_string())
        );

        assert_eq!(Type::Integer(-1), <i8 as Into<Type>>::into(-1_i8));
        assert_eq!(Type::Integer(-2), <i16 as Into<Type>>::into(-2_i16));
        assert_eq!(Type::Integer(-3), <i32 as Into<Type>>::into(-3_i32));
        assert_eq!(Type::Integer(-4), <i64 as Into<Type>>::into(-4_i64));

        assert_eq!(Type::PositiveInteger(1), <u8 as Into<Type>>::into(1_u8));
        assert_eq!(Type::PositiveInteger(2), <u16 as Into<Type>>::into(2_u16));
        assert_eq!(Type::PositiveInteger(3), <u32 as Into<Type>>::into(3_u32));
        assert_eq!(Type::PositiveInteger(4), <u64 as Into<Type>>::into(4_u64));

        assert_eq!(Type::Number(123.5), <f32 as Into<Type>>::into(123.5_f32));
        assert_eq!(Type::Number(123.4), <f64 as Into<Type>>::into(123.4_f64));

        assert_eq!(Type::Boolean(true), <bool as Into<Type>>::into(true));
        assert_eq!(Type::Boolean(false), <bool as Into<Type>>::into(false));

        assert_eq!(
            Type::Time(NaiveTime::from_str("12:34").unwrap()),
            <NaiveTime as Into<Type>>::into(NaiveTime::from_str("12:34").unwrap())
        );
        assert_eq!(
            Type::Date(NaiveDate::from_str("2026-08-01").unwrap()),
            <NaiveDate as Into<Type>>::into(NaiveDate::from_str("2026-08-01").unwrap())
        );

        assert_eq!(
            Type::GridSquare(GridSquare::new("JO30").unwrap()),
            <GridSquare as Into<Type>>::into(GridSquare::new("JO30").unwrap())
        );
    }

    #[test]
    fn test_300_record_date_time() {
        // Without any date/time
        let mut rec = Record::from(vec![("CALL", "DK5XXX"), ("NAME", "Chris")]);
        assert_eq!(None, rec.date_time_on());
        assert_eq!(None, rec.date_time_off());

        // With TIME_ON
        rec.insert_str("TIME_ON", "1245");
        assert_eq!(None, rec.date_time_on());
        assert_eq!(None, rec.date_time_off());

        // With QSO_DATE and TIME_ON
        rec.insert_str("QSO_DATE", "20231009");
        assert_eq!(
            Some(DateTime::<Utc>::from_str("2023-10-09T12:45:00Z").unwrap()),
            rec.date_time_on()
        );
        assert_eq!(None, rec.date_time_off());

        // With TIME_OFF
        rec.insert_str("TIME_OFF", "1250");
        assert_eq!(
            Some(DateTime::<Utc>::from_str("2023-10-09T12:45:00Z").unwrap()),
            rec.date_time_on()
        );
        assert_eq!(
            Some(DateTime::<Utc>::from_str("2023-10-09T12:50:00Z").unwrap()),
            rec.date_time_off()
        );

        // With QSO_DATE_OFF and TIME_OFF
        rec.insert_str("QSO_DATE_OFF", "20231010");
        assert_eq!(
            Some(DateTime::<Utc>::from_str("2023-10-09T12:45:00Z").unwrap()),
            rec.date_time_on()
        );
        assert_eq!(
            Some(DateTime::<Utc>::from_str("2023-10-10T12:50:00Z").unwrap()),
            rec.date_time_off()
        );
    }

    #[test]
    fn test_400_userdef_deser_adi() {
        let adi = "Comment\n\
        <CREATED_TIMESTAMP:13>20250705 1234 <ADIF_VER:5>3.1.4 <PROGRAMID:4>Test \
        <PROGRAMVERSION:3>3.2 <userDEF1:3:N>XXX <USERdef2:12:N>Test,{1,2,3} <USERDEF3:12>Txxt,{1,2,3} <EOH>\n";

        let mut doc = Doc::new();
        doc.deserialize_adi(adi).unwrap();
        assert_eq!(doc.header["USERDEF1"].to_string(), "XXX, type N");
        assert_eq!(doc.header["USERDEF2"].to_string(), "Test,{1,2,3}, type N");
        assert_eq!(doc.header["USERDEF3"].to_string(), "Txxt,{1,2,3}, type ");
    }

    #[test]
    fn test_410_userdef_ser_adi() {
        let mut hdr = Header::new();
        hdr.insert(
            "USERDEF",
            Type::UserDef {
                field_type: "N".to_string(),
                definition: "SUN_VALUE".to_string(),
            },
        );
        hdr.insert(
            "USERDEF1",
            Type::UserDef {
                field_type: "B".to_string(),
                definition: "SUN_SHINED".to_string(),
            },
        );
        hdr.insert(
            "USERDEF2",
            Type::UserDef {
                field_type: "".to_string(),
                definition: "TEST".to_string(),
            },
        );
        assert_eq!(
            hdr.serialize_adi(),
            "\n<USERDEF:9:N>SUN_VALUE\n<USERDEF1:10:B>SUN_SHINED\n<USERDEF2:4>TEST\n<EOH>"
        );
    }

    #[test]
    fn test_450_app_deser_adi() {
        // APP_ fields in header are not compliant to the standard but somewhat useful
        let adi = "Comment\n\
        <CREATED_TIMESTAMP:13>20250705 1234 <ADIF_VER:5>3.1.4 <PROGRAMID:4>Test \
        <PROGRAMVERSION:3>3.2 <app_TEST_xxx:3:N>4.5 <app_TEST_YYY:4>Test \
        <app_TEST_last_used_date:8:d>20260614 <app_TEST_last_used_time:4:t>1559 <EOH>\n\
        <QSO_DATE:8>20231009 <TIME_ON:4>1245 <CALL:6>DK5ABC <NAME:5>Chris \
        <APP_TEST_Friend:1:b>y <APP_TEST_SINCE:5:D>12345 <APP_TEST_guest:1:b>o <APP_TEST_member:1:b>n <eor>\n\n";

        let mut doc = Doc::new();
        doc.deserialize_adi(adi).unwrap();
        assert_eq!(doc.header["APP_TEST_XXX"], Type::Number(4.5));
        assert_eq!(doc.header["APP_TEST_YYY"], Type::String("Test".to_string()));
        assert_eq!(
            doc.header["APP_TEST_LAST_USED_DATE"],
            Type::Date(NaiveDate::from_str("2026-06-14").unwrap())
        );
        assert_eq!(
            doc.header["APP_TEST_LAST_USED_TIME"],
            Type::Time(NaiveTime::from_str("15:59").unwrap())
        );
        assert_eq!(doc.records[0]["APP_TEST_FRIEND"], Type::Boolean(true));
        assert_eq!(
            doc.records[0]["APP_TEST_SINCE"],
            Type::String("12345".to_string())
        );
        assert_eq!(doc.records[0]["APP_TEST_GUEST"], Type::Boolean(false));
        assert_eq!(doc.records[0]["APP_TEST_MEMBER"], Type::Boolean(false));
    }

    #[test]
    fn test_460_app_ser_adi() {
        let mut hdr = Header::new();
        hdr.insert("APP_TEST_XXX", Type::Number(4.5));
        hdr.insert("APP_TEST_YYY", Type::String("Test".to_string()));
        hdr.insert(
            "APP_TEST_LAST_USED_DATE",
            Type::Date(NaiveDate::from_str("2026-06-14").unwrap()),
        );
        hdr.insert(
            "APP_TEST_LAST_USED_TIME",
            Type::Time(NaiveTime::from_str("15:59").unwrap()),
        );
        assert_eq!(
            hdr.serialize_adi(),
            "\n<APP_TEST_XXX:3:N>4.5\n<APP_TEST_YYY:4:S>Test\n<APP_TEST_LAST_USED_DATE:8:D>20260614\n<APP_TEST_LAST_USED_TIME:6:T>155900\n<EOH>"
        );

        let mut qso = Record::new();
        qso.insert("APP_TEST_FRIEND", Type::Boolean(true));
        qso.insert("APP_TEST_SINCE", Type::String("12345".to_string()));
        assert_eq!(
            qso.serialize_adi(),
            "<APP_TEST_FRIEND:1:B>Y <APP_TEST_SINCE:5:S>12345\n<EOR>"
        );
    }

    #[test]
    #[cfg(feature = "serde_loose")]
    fn test_500_serde_loose_ser() {
        let adi = "Comment\n\
        <CREATED_TIMESTAMP:13>20250705 1234 <ADIF_VER:5>3.1.4 <PROGRAMID:4>Test \
        <PROGRAMVERSION:3>3.2 <USERDEF1:3:N>XXX <EOH>\n\
        <QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>DL4BDF <NAME:6>Walter \
        <DISTANCE:5>123.4 <QSO_RANDOM:1>y <SWL:1>n <GRIDSQUARE:6>JO35er <MY_GRIDSQUARE:6>JO30ui \
        <NR_BURSTS:1>9 <CQZ:3>123 <EOR>\n\
        <QSO_DATE:8>20231009 <TIME_ON:4>0235 <CALL:6>DO5BDI <NAME:5>Heinz \
        <DISTANCE:5>456.7 <QSO_RANDOM:1>N <SWL:1>Y <GRIDSQUARE:6>JO31rr <MY_GRIDSQUARE:6>JO30ui \
        <NR_BURSTS:1>7 <CQZ:3>456 <EOR>";

        let json = "{\
        \"header\":{\
            \"record\":{\
                \"CREATED_TIMESTAMP\":\"20250705 1234\",\"ADIF_VER\":\"3.1.4\",\
                \"PROGRAMID\":\"Test\",\"PROGRAMVERSION\":\"3.2\",\"USERDEF1\":\"XXX\"},\
            \"comment\":\"Comment\"},\
        \"records\":[\
            {\
                \"QSO_DATE\":\"20231008\",\"TIME_ON\":\"114500\",\"CALL\":\"DL4BDF\",\
                \"NAME\":\"Walter\",\"DISTANCE\":123.4,\"QSO_RANDOM\":true,\"SWL\":false,\
                \"GRIDSQUARE\":\"JO35er\",\"MY_GRIDSQUARE\":\"JO30ui\",\"NR_BURSTS\":9,\"CQZ\":123},\
            {\
                \"QSO_DATE\":\"20231009\",\"TIME_ON\":\"023500\",\"CALL\":\"DO5BDI\",\
                \"NAME\":\"Heinz\",\"DISTANCE\":456.7,\"QSO_RANDOM\":false,\"SWL\":true,\
                \"GRIDSQUARE\":\"JO31rr\",\"MY_GRIDSQUARE\":\"JO30ui\",\"NR_BURSTS\":7,\"CQZ\":456}\
            ]}";

        let mut doc = Doc::new();
        doc.deserialize_adi(adi).unwrap();
        assert_eq!(
            serde_json::to_string(&doc).expect("could not serialize to JSON"),
            json
        );
    }

    #[test]
    #[cfg(feature = "serde_loose")]
    fn test_510_serde_loose_de() {
        let adi = "Comment\n\
        <CREATED_TIMESTAMP:13>20250705 1234\n<ADIF_VER:5>3.1.4\n<PROGRAMID:4>Test\n\
        <PROGRAMVERSION:3>3.2\n<USERDEF1:3>XXX\n<EOH>\n\n\
        <QSO_DATE:8>20231008 <TIME_ON:6>114500 <CALL:6>DL4BDF <NAME:6>Walter <DISTANCE:5>123.4\n\
        <QSO_RANDOM:1>Y <SWL:1>N <GRIDSQUARE:6>JO35er <MY_GRIDSQUARE:6>JO30ui <NR_BURSTS:1>9\n\
        <CQZ:3>123\n\
        <EOR>\n\n\
        <QSO_DATE:8>20231009 <TIME_ON:6>023500 <CALL:6>DO5BDI <NAME:5>Heinz <DISTANCE:5>456.7\n\
        <QSO_RANDOM:1>N <SWL:1>Y <GRIDSQUARE:6>JO31rr <MY_GRIDSQUARE:6>JO30ui <NR_BURSTS:1>7\n\
        <CQZ:3>456\n\
        <EOR>";

        let json = "{\
        \"header\":{\
            \"record\":{\
                \"CREATED_TIMESTAMP\":\"20250705 1234\",\"ADIF_VER\":\"3.1.4\",\
                \"PROGRAMID\":\"Test\",\"PROGRAMVERSION\":\"3.2\",\"USERDEF1\":\"XXX\"},\
            \"comment\":\"Comment\"},\
        \"records\":[\
            {\
                \"QSO_DATE\":\"20231008\",\"TIME_ON\":\"114500\",\"CALL\":\"DL4BDF\",\
                \"NAME\":\"Walter\",\"DISTANCE\":123.4,\"QSO_RANDOM\":true,\"SWL\":false,\
                \"GRIDSQUARE\":\"JO35er\",\"MY_GRIDSQUARE\":\"JO30ui\",\"NR_BURSTS\":9,\"CQZ\":123},\
            {\
                \"QSO_DATE\":\"20231009\",\"TIME_ON\":\"023500\",\"CALL\":\"DO5BDI\",\
                \"NAME\":\"Heinz\",\"DISTANCE\":456.7,\"QSO_RANDOM\":false,\"SWL\":true,\
                \"GRIDSQUARE\":\"JO31rr\",\"MY_GRIDSQUARE\":\"JO30ui\",\"NR_BURSTS\":7,\"CQZ\":456}\
            ]}";

        let doc: Doc = serde_json::from_str(&json).expect("could not deserialize from JSON");
        assert_eq!(doc.serialize_adi(), adi);
    }

    #[test]
    #[cfg(feature = "serde_impl")]
    fn test_550_serde_impl_ser() {
        let adi = "Comment\n\
        <CREATED_TIMESTAMP:13>20250705 1234 <ADIF_VER:5>3.1.4 <PROGRAMID:4>Test \
        <PROGRAMVERSION:3>3.2 <USERDEF1:3:N>XXX <APP_TEST_LAST_USED_DATE:8:D>20260614\n<EOH>\n\
        <QSO_DATE:8>20231008 <TIME_ON:4>1145 <CALL:6>DL4BDF <NAME:6>Walter \
        <DISTANCE:5>123.4 <QSO_RANDOM:1>y <SWL:1>n <GRIDSQUARE:6>JO35er <MY_GRIDSQUARE:6>JO30ui \
        <NR_BURSTS:1>9 <CQZ:3>123 <APP_TEST_FRIEND:1:B>Y <EOR>\n\
        <QSO_DATE:8>20231009 <TIME_ON:4>0235 <CALL:6>DO5BDI <NAME:5>Heinz \
        <DISTANCE:5>456.7 <QSO_RANDOM:1>N <SWL:1>Y <GRIDSQUARE:6>JO31rr <MY_GRIDSQUARE:6>JO30ui \
        <NR_BURSTS:1>7 <CQZ:3>456 <APP_TEST_FRIEND:1:B>N <EOR>";

        let json = "{\
        \"header\":{\
            \"record\":{\
                \"CREATED_TIMESTAMP\":{\"String\":\"20250705 1234\"},\"ADIF_VER\":{\"String\":\"3.1.4\"},\"PROGRAMID\":{\"String\":\"Test\"},\"PROGRAMVERSION\":{\"String\":\"3.2\"},\"USERDEF1\":{\"UserDef\":{\"field_type\":\"N\",\"definition\":\"XXX\"}},\"APP_TEST_LAST_USED_DATE\":{\"Date\":\"2026-06-14\"}},\
                \"comment\":\"Comment\"\
                },\
        \"records\":[\
            {\"QSO_DATE\":{\"Date\":\"2023-10-08\"},\"TIME_ON\":{\"Time\":\"11:45:00\"},\"CALL\":{\"String\":\"DL4BDF\"},\"NAME\":{\"String\":\"Walter\"},\"DISTANCE\":{\"Number\":123.4},\"QSO_RANDOM\":{\"Boolean\":true},\"SWL\":{\"Boolean\":false},\"GRIDSQUARE\":{\"GridSquare\":\"JO35er\"},\"MY_GRIDSQUARE\":{\"GridSquare\":\"JO30ui\"},\"NR_BURSTS\":{\"Integer\":9},\"CQZ\":{\"PositiveInteger\":123},\"APP_TEST_FRIEND\":{\"Boolean\":true}},\
            {\"QSO_DATE\":{\"Date\":\"2023-10-09\"},\"TIME_ON\":{\"Time\":\"02:35:00\"},\"CALL\":{\"String\":\"DO5BDI\"},\"NAME\":{\"String\":\"Heinz\"},\"DISTANCE\":{\"Number\":456.7},\"QSO_RANDOM\":{\"Boolean\":false},\"SWL\":{\"Boolean\":true},\"GRIDSQUARE\":{\"GridSquare\":\"JO31rr\"},\"MY_GRIDSQUARE\":{\"GridSquare\":\"JO30ui\"},\"NR_BURSTS\":{\"Integer\":7},\"CQZ\":{\"PositiveInteger\":456},\"APP_TEST_FRIEND\":{\"Boolean\":false}}\
            ]}";

        let mut doc = Doc::new();
        doc.deserialize_adi(adi).unwrap();
        assert_eq!(
            serde_json::to_string(&doc).expect("could not serialize to JSON"),
            json
        );
    }

    #[test]
    #[cfg(feature = "serde_impl")]
    fn test_560_serde_impl_de() {
        let adi = "Comment\n\
        <CREATED_TIMESTAMP:13>20250705 1234\n<ADIF_VER:5>3.1.4\n<PROGRAMID:4>Test\n\
        <PROGRAMVERSION:3>3.2\n<USERDEF1:3:N>XXX\n<APP_TEST_LAST_USED_DATE:8:D>20260614\n<EOH>\n\n\
        <QSO_DATE:8>20231008 <TIME_ON:6>114500 <CALL:6>DL4BDF <NAME:6>Walter <DISTANCE:5>123.4\n\
        <QSO_RANDOM:1>Y <SWL:1>N <GRIDSQUARE:6>JO35er <MY_GRIDSQUARE:6>JO30ui <NR_BURSTS:1>9\n\
        <CQZ:3>123 <APP_TEST_FRIEND:1:B>Y\n<EOR>\n\n\
        <QSO_DATE:8>20231009 <TIME_ON:6>023500 <CALL:6>DO5BDI <NAME:5>Heinz <DISTANCE:5>456.7\n\
        <QSO_RANDOM:1>N <SWL:1>Y <GRIDSQUARE:6>JO31rr <MY_GRIDSQUARE:6>JO30ui <NR_BURSTS:1>7\n\
        <CQZ:3>456 <APP_TEST_FRIEND:1:B>N\n<EOR>";

        let json = "{\
        \"header\":{\
            \"record\":{\
                \"CREATED_TIMESTAMP\":{\"String\":\"20250705 1234\"},\"ADIF_VER\":{\"String\":\"3.1.4\"},\"PROGRAMID\":{\"String\":\"Test\"},\"PROGRAMVERSION\":{\"String\":\"3.2\"},\"USERDEF1\":{\"UserDef\":{\"field_type\":\"N\",\"definition\":\"XXX\"}},\"APP_TEST_LAST_USED_DATE\":{\"Date\":\"2026-06-14\"}},\
                \"comment\":\"Comment\"\
                },\
        \"records\":[\
            {\"QSO_DATE\":{\"Date\":\"2023-10-08\"},\"TIME_ON\":{\"Time\":\"11:45:00\"},\"CALL\":{\"String\":\"DL4BDF\"},\"NAME\":{\"String\":\"Walter\"},\"DISTANCE\":{\"Number\":123.4},\"QSO_RANDOM\":{\"Boolean\":true},\"SWL\":{\"Boolean\":false},\"GRIDSQUARE\":{\"GridSquare\":\"JO35er\"},\"MY_GRIDSQUARE\":{\"String\":\"JO30ui\"},\"NR_BURSTS\":{\"Integer\":9},\"CQZ\":{\"PositiveInteger\":123},\"APP_TEST_FRIEND\":{\"Boolean\":true}},\
            {\"QSO_DATE\":{\"Date\":\"2023-10-09\"},\"TIME_ON\":{\"Time\":\"02:35:00\"},\"CALL\":{\"String\":\"DO5BDI\"},\"NAME\":{\"String\":\"Heinz\"},\"DISTANCE\":{\"Number\":456.7},\"QSO_RANDOM\":{\"Boolean\":false},\"SWL\":{\"Boolean\":true},\"GRIDSQUARE\":{\"GridSquare\":\"JO31rr\"},\"MY_GRIDSQUARE\":{\"String\":\"JO30ui\"},\"NR_BURSTS\":{\"Integer\":7},\"CQZ\":{\"PositiveInteger\":456},\"APP_TEST_FRIEND\":{\"Boolean\":false}}\
            ]}";

        let doc: Doc = serde_json::from_str(&json).expect("could not deserialize from JSON");
        assert_eq!(doc.serialize_adi(), adi);
    }
}

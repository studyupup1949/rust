pub mod data_type_def_xs;
pub mod lang_string;

use crate::utilities::deserialize_normalized_text;
use crate::part_1::v3_1::LangString;
use serde::{Deserialize, Serialize};

// TODO: Base64 parsing
pub type BlobType = Vec<u8>;

// TODO: Mime Parsing?
pub type ContentType = String;

pub type DateTimeUTC = chrono::DateTime<chrono::Utc>;

// Min 1, Max 2048
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Identifier(#[serde(deserialize_with = "deserialize_normalized_text")] String);

// TODO: Min 1, Max 64
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct LabelType(String);

pub type LangStringSet = Vec<LangString>;
pub type MultiLanguageNameType = LangStringSet;

// TODO: Min 1, Max 255
pub type MessageTopicType = String;

pub type Uri = iref::UriBuf;
pub type Iri = iref::IriBuf;

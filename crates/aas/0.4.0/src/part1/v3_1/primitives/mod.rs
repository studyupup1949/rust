pub mod data_type_def_xs;
mod identifier;
mod label;
pub mod lang_string;
mod message_topic;

pub use identifier::*;
pub use label::*;
pub use message_topic::*;

use crate::part1::v3_1::LangString;

// TODO: Base64 parsing
pub type BlobType = Vec<u8>;

// TODO: Mime Parsing?
pub type ContentType = String;

pub type DateTimeUTC = iso8601::DateTime;

pub type LangStringSet = Vec<LangString>;

pub type MultiLanguageNameType = LangStringSet;

// UriBuf/IriBuf or UriRefBuf/IriRefBuf?

pub type Uri = iref::UriRefBuf;
pub type Iri = iref::IriRefBuf;

pub(crate) mod xml {
    use crate::part1::v3_1::primitives::MultiLanguageNameType;
    use serde::{Deserialize, Serialize};

    // needed for e.g.
    // <displayName>
    //      <langStringTextType>...</langStringTextType>
    //      <langStringTextType>...</langStringTextType>
    // </displayName>
    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct LangStringTextType {
        #[serde(rename = "$value")]
        pub(crate) values: MultiLanguageNameType,
    }

    impl From<MultiLanguageNameType> for LangStringTextType {
        fn from(value: MultiLanguageNameType) -> Self {
            Self { values: value }
        }
    }
    impl From<LangStringTextType> for MultiLanguageNameType {
        fn from(value: LangStringTextType) -> Self {
            value.values
        }
    }
}

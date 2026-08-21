use crate::part_1::v3_1::key::Key;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use strum::EnumString;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct ReferenceInner {
    /// E.g. semantic id of a standard submodel
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "referredSemanticId")]
    pub referred_semantic_id: Option<Box<Reference>>,

    pub keys: Vec<Key>,
}

/// A `Reference` is a mechanism to unambiguously identify one or multiple elements within or across
/// Asset Administration Shells. It consists of an ordered list of `Keys`, where each `Key` defines
/// a single step in the reference path, targeting specific sub-elements or external resources.
///
/// The `ReferenceType` attribute of a `Reference` determines the scope and semantics of the reference:
/// - `GlobalReference` means the reference resolves to an element identifiable globally,
/// often outside the local AAS context.
/// - `LocalReference` restricts the reference scope to internal elements or fragments
/// within the current parent element or AAS.
///
/// This distinction affects how references are interpreted, resolved, and validated in distributed environments,
/// ensuring interoperability and correct addressing in digital twin ecosystems.
///
/// A `Reference` supports multi-level navigation through composite structures by chaining multiple keys,
/// enabling precise targeting of nested submodels, submodel elements, or fragments.
#[derive(EnumString, Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Reference {
    ExternalReference(ReferenceInner),
    ModelReference(ReferenceInner),
}

impl ReferenceInner {
    pub fn new(key: Key) -> Self {
        Self {
            referred_semantic_id: None,
            keys: vec![key],
        }
    }

    pub fn from_vec(keys: Vec<Key>) -> Self {
        Self {
            referred_semantic_id: None,
            keys,
        }
    }
}

pub fn deserialize_model_reference<'de, D>(deserializer: D) -> Result<Reference, D::Error>
where
    D: Deserializer<'de>,
{
    // Define a visitor to handle deserialization from sequence of ModelReference
    struct SubmodelsVisitor;

    impl<'de> Visitor<'de> for SubmodelsVisitor {
        type Value = Reference;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional sequence of ModelReference")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let reference: Reference = Reference::deserialize(deserializer)?;

            if let Reference::ModelReference(model_ref_inner) = reference {
                // Convert model_ref_inner into Submodel
                // Assuming From<ReferenceInner> for Submodel is implemented
                Ok(Reference::ModelReference(model_ref_inner))
            } else {
                Err(D::Error::custom("unexpected reference type"))
            }
        }
    }

    deserializer.deserialize_option(SubmodelsVisitor)
}

pub fn deserialize_external_reference<'de, D>(deserializer: D) -> Result<Reference, D::Error>
where
    D: Deserializer<'de>,
{
    // Define a visitor to handle deserialization from sequence of ModelReference
    struct SubmodelsVisitor;

    impl<'de> Visitor<'de> for SubmodelsVisitor {
        type Value = Reference;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional sequence of ModelReference")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let reference: Reference = Reference::deserialize(deserializer)?;

            if let Reference::ExternalReference(model_ref_inner) = reference {
                // Convert model_ref_inner into Submodel
                // Assuming From<ReferenceInner> for Submodel is implemented
                Ok(Reference::ExternalReference(model_ref_inner))
            } else {
                Err(D::Error::custom("unexpected reference type"))
            }
        }
    }

    deserializer.deserialize_option(SubmodelsVisitor)
}

pub fn deserialize_optional_model_reference<'de, D>(
    deserializer: D,
) -> Result<Option<Reference>, D::Error>
where
    D: Deserializer<'de>,
{
    // Define a visitor to handle deserialization from sequence of ModelReference
    struct SubmodelsVisitor;

    impl<'de> Visitor<'de> for SubmodelsVisitor {
        type Value = Option<Reference>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional sequence of ModelReference")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let reference: Reference = Reference::deserialize(deserializer)?;

            if let Reference::ModelReference(model_ref_inner) = reference {
                // Convert model_ref_inner into Submodel
                // Assuming From<ReferenceInner> for Submodel is implemented
                Ok(Some(Reference::ModelReference(model_ref_inner)))
            } else {
                Err(D::Error::custom("unexpected reference type"))
            }
        }
    }

    deserializer.deserialize_option(SubmodelsVisitor)
}

pub fn deserialize_optional_external_reference<'de, D>(
    deserializer: D,
) -> Result<Option<Reference>, D::Error>
where
    D: Deserializer<'de>,
{
    // Define a visitor to handle deserialization from sequence of ModelReference
    struct SubmodelsVisitor;

    impl<'de> Visitor<'de> for SubmodelsVisitor {
        type Value = Option<Reference>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional sequence of ModelReference")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let reference: Reference = Reference::deserialize(deserializer)?;

            if let Reference::ExternalReference(model_ref_inner) = reference {
                // Convert model_ref_inner into Submodel
                // Assuming From<ReferenceInner> for Submodel is implemented
                Ok(Some(Reference::ExternalReference(model_ref_inner)))
            } else {
                Err(D::Error::custom("unexpected reference type"))
            }
        }
    }

    deserializer.deserialize_option(SubmodelsVisitor)
}

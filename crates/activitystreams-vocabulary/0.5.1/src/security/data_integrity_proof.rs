use serde::{Deserialize, Serialize};

use crate::{
    Context, DateTime, Error, Iri, MultibaseData, Result, SecurityType, VocabularyTypes,
    derived_kind_serde, field_access, impl_default, impl_display,
};

/// Represents a data-integrity proof [cryptosuite](https://www.w3.org/TR/vc-di-eddsa/#instantiate-cryptosuite).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Cryptosuite {
    #[serde(rename = "eddsa-jcs-2022")]
    EddsaJcs2022,
    #[serde(rename = "eddsa-rdfc-2022")]
    EddsaRdfc2022,
}

impl Cryptosuite {
    pub const EDDSA_JCS_2022: &str = "eddsa-jcs-2022";
    pub const EDDSA_RDFC_2022: &str = "eddsa-rdfc-2022";

    /// Creates a new [Cryptosuite].
    pub const fn new() -> Self {
        Self::EddsaJcs2022
    }

    /// Gets the [Cryptosuite] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EddsaJcs2022 => Self::EDDSA_JCS_2022,
            Self::EddsaRdfc2022 => Self::EDDSA_RDFC_2022,
        }
    }
}

impl_default!(Cryptosuite);
impl_display!(Cryptosuite, str);

/// Represents a [DataIntegrityProof](https://www.w3.org/TR/vc-data-integrity/#dataintegrityproof).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataIntegrityProof {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    context_property: Option<Context>,
    #[serde(
        rename = "type",
        deserialize_with = "obj_serde::de",
        serialize_with = "obj_serde::ser"
    )]
    kind: VocabularyTypes,
    cryptosuite: Cryptosuite,
    proof_value: MultibaseData,
    proof_purpose: Iri,
    created: DateTime,
    verification_method: Iri,
}

impl DataIntegrityProof {
    /// Creates a new [DataIntegrityProof].
    pub fn new() -> Self {
        Self {
            context_property: None,
            kind: SecurityType::DataIntegrityProof.into(),
            cryptosuite: Cryptosuite::new(),
            proof_value: MultibaseData::new(),
            proof_purpose: Iri::new(),
            created: DateTime::default(),
            verification_method: Iri::new(),
        }
    }

    /// Creates a new [DataIntegrityProof] without the `@context` property field.
    pub fn new_inner() -> Self {
        Self::new().without_context_property()
    }

    /// Builder function that unsets the `@context` property field.
    pub fn without_context_property(self) -> Self {
        Self {
            context_property: None,
            ..self
        }
    }

    /// Gets the data integrity proof bytes.
    pub fn proof_bytes(&self) -> Result<DataIntegrityProofBytes> {
        DataIntegrityProofBytes::from_bytes(self.cryptosuite, self.proof_value.data())
    }
}

derived_kind_serde!(SecurityType, DataIntegrityProof);
impl_default!(DataIntegrityProof);
impl_display!(DataIntegrityProof, json);

field_access! {
    DataIntegrityProof {
        context_property: option_ref { Context },
    }
}

field_access! {
    DataIntegrityProof {
        kind: as_ref { VocabularyTypes },
        proof_value: as_ref { MultibaseData },
        proof_purpose: as_ref { Iri },
        created: as_ref { DateTime },
        verification_method: as_ref { Iri },
    }
}

field_access! {
    DataIntegrityProof {
        cryptosuite: Cryptosuite,
    }
}

/// Represents [data integrity proof](https://www.w3.org/TR/vc-data-intregrity/#dfn-data-integrity-proof) bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataIntegrityProofBytes {
    EddsaJcs2022([u8; 64]),
    EddsaRdfc2022([u8; 64]),
}

impl DataIntegrityProofBytes {
    pub const EDDSA_JCS_2022_LEN: usize = 64;
    pub const EDDSA_RDFC_2022_LEN: usize = 64;

    /// Creates a new [DataIntegrityProofBytes].
    pub const fn new() -> Self {
        Self::EddsaJcs2022([0u8; Self::EDDSA_JCS_2022_LEN])
    }

    /// Gets a reference to the proof bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::EddsaJcs2022(bytes) => bytes,
            Self::EddsaRdfc2022(bytes) => bytes,
        }
    }

    /// Converts the [DataIntegrityProofBytes] into a byte vector.
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            Self::EddsaJcs2022(bytes) => bytes.to_vec(),
            Self::EddsaRdfc2022(bytes) => bytes.to_vec(),
        }
    }

    /// Attempts to convert a byte slice into [DataIntegrityProofBytes].
    pub fn from_bytes<A: AsRef<[u8]>>(cryptosuite: Cryptosuite, val: A) -> Result<Self> {
        match cryptosuite {
            Cryptosuite::EddsaJcs2022 => <[u8; Self::EDDSA_JCS_2022_LEN]>::try_from(val.as_ref())
                .map(Self::EddsaJcs2022)
                .map_err(|err| {
                    Error::multikey(format!("invalid eddsa-jcs-2022 proof length: {err}"))
                }),
            Cryptosuite::EddsaRdfc2022 => <[u8; Self::EDDSA_JCS_2022_LEN]>::try_from(val.as_ref())
                .map(Self::EddsaRdfc2022)
                .map_err(|err| {
                    Error::multikey(format!("invalid eddsa-rdfc-2022 proof length: {err}"))
                }),
        }
    }
}

impl_default!(DataIntegrityProofBytes);

impl core::fmt::Display for DataIntegrityProofBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        serde_json::to_string(self.as_bytes())
            .map_err(|_| core::fmt::Error)
            .and_then(|s| write!(f, "{s}"))
    }
}

impl<'a> From<&'a DataIntegrityProofBytes> for &'a [u8] {
    fn from(val: &'a DataIntegrityProofBytes) -> Self {
        val.as_bytes()
    }
}

impl From<DataIntegrityProofBytes> for Vec<u8> {
    fn from(val: DataIntegrityProofBytes) -> Self {
        val.to_bytes()
    }
}

impl<A: AsRef<[u8]>> TryFrom<(Cryptosuite, A)> for DataIntegrityProofBytes {
    type Error = Error;

    fn try_from(val: (Cryptosuite, A)) -> Result<Self> {
        let (suite, bytes) = val;
        Self::from_bytes(suite, bytes)
    }
}

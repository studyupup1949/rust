use thiserror::Error;

/// type definitions for version 3.1.1 of the AAS Specification part 1.
/// <https://industrialdigitaltwin.io/aas-specifications/IDTA-01001/v3.1.1/index.html>
pub mod v3_1;

/// see https://industrialdigitaltwin.io/aas-specifications/IDTA-01001/v3.1.1/mappings/mappings.html#value-only-serialization-in-json
pub trait ToJsonValue {
    type Error;
    fn to_json_value(&self) -> Result<String, Self::Error>;
}

/// see https://industrialdigitaltwin.io/aas-specifications/IDTA-01001/v3.1.1/mappings/mappings.html#_format_metadata_metadata_serialization
pub trait ToJsonMetamodel {
    type Error;
    fn to_json_metamodel(&self) -> Result<String, Self::Error>;
}

#[derive(Debug, Error)]
pub enum MetamodelError {
    #[error(transparent)]
    FailedDeserialisation(serde_json::Error),

    #[error(transparent)]
    FailedSerialisation(serde_json::Error),

    #[error("Struct does not support json metamodel")]
    MetamodelNotSupported,

    #[error("Struct does not support json value format")]
    ValueFormatNotSupported,
}

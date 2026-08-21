use crate::part_1::v3_1::level_type::LevelType;
use crate::part_1::v3_1::primitives::lang_string::LangString;
use crate::part_1::v3_1::reference::Reference;
use crate::part_1::v3_1::reference::deserialize_external_reference;
use crate::part_1::v3_1::value_list::ValueList;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// HasDataSpecification
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct HasDataSpecification {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "embeddedDataSpecifications")]
    embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EmbeddedDataSpecification {
    #[serde(rename = "dataSpecification")]
    #[serde(deserialize_with = "deserialize_external_reference")]
    pub data_specification: Reference,

    #[serde(rename = "dataSpecificationContent")]
    pub data_specification_content: DataSpecificationIec61360,
}

// THIS IS PART 3. TODO?
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(tag = "modelType")]
pub struct DataSpecificationIec61360 {
    #[serde(rename = "preferredName")]
    pub preferred_name: Vec<LangString>,

    #[serde(rename = "shortName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<Vec<LangString>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    #[serde(rename = "unitId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_id: Option<Reference>,

    #[serde(rename = "sourceOfDefinition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_definition: Option<String>,

    #[serde(rename = "symbol")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    #[serde(rename = "dataType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataTypeIec61360>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<LangString>,

    #[serde(rename = "valueFormat")]
    pub value_format: Option<String>,

    #[serde(rename = "valueList")]
    pub value_list: Option<Vec<ValueList>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "levelType")]
    pub level_type: Option<LevelType>,
}

#[derive(EnumString, Display, Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum DataTypeIec61360 {
    #[serde(rename = "BLOB")]
    Blob,
    #[serde(rename = "BOOLEAN")]
    Boolean,
    #[serde(rename = "DATE")]
    Date,
    #[serde(rename = "FILE")]
    File,
    #[serde(rename = "HTML")]
    Html,
    #[serde(rename = "INTEGER_COUNT")]
    IntegerCount,
    #[serde(rename = "INTEGER_CURRENCY")]
    IntegerCurrency,
    #[serde(rename = "INTEGER_MEASURE")]
    IntegerMeasure,
    #[serde(rename = "IRDI")]
    Irdi,
    #[serde(rename = "IRI")]
    Iri,
    #[serde(rename = "RATIONAL")]
    Rational,
    #[serde(rename = "RATIONAL_MEASURE")]
    RationalMeasure,
    #[serde(rename = "REAL_COUNT")]
    RealCount,
    #[serde(rename = "REAL_CURRENCY")]
    RealCurrency,
    #[serde(rename = "REAL_MEASURE")]
    RealMeasure,
    #[serde(rename = "STRING")]
    String,
    #[serde(rename = "STRING_TRANSLATABLE")]
    StringTranslatable,
    #[serde(rename = "TIME")]
    Time,
    #[serde(rename = "TIMESTAMP")]
    Timestamp,
}

#[cfg(test)]
mod tests {

    #[ignore]
    #[test]
    fn test_submodel_elements() {
        todo!()
    }
}

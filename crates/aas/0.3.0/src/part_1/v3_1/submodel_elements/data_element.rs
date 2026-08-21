use crate::part_1::v3_1::submodel_elements::Blob;
use crate::part_1::v3_1::submodel_elements::file::File;
use crate::part_1::v3_1::submodel_elements::multi_language_property::MultiLanguageProperty;
use crate::part_1::v3_1::submodel_elements::property::Property;
use crate::part_1::v3_1::submodel_elements::range::Range;
use crate::part_1::v3_1::submodel_elements::reference_element::ReferenceElement;
use crate::part_1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
use strum::Display;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum DataElement {
    Blob(Blob),
    File(File),
    MultiLanguageProperty(MultiLanguageProperty),
    Property(Property),
    Range(Range),
    ReferenceElement(ReferenceElement),
}

impl ToJsonMetamodel for DataElement {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        match self {
            DataElement::Blob(element) => element.to_json_metamodel(),
            DataElement::File(element) => element.to_json_metamodel(),
            DataElement::MultiLanguageProperty(element) => element.to_json_metamodel(),
            DataElement::Property(element) => element.to_json_metamodel(),
            DataElement::Range(element) => Ok(element.to_json_metamodel().unwrap()),
            DataElement::ReferenceElement(element) => Ok(element.to_json_metamodel().unwrap()),
        }
    }
}

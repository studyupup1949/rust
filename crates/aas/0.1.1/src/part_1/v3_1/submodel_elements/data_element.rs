use crate::part_1::v3_1::submodel_elements::Blob;
use crate::part_1::v3_1::submodel_elements::file::File;
use crate::part_1::v3_1::submodel_elements::multi_language_property::MultiLanguageProperty;
use crate::part_1::v3_1::submodel_elements::property::Property;
use crate::part_1::v3_1::submodel_elements::range::Range;
use crate::part_1::v3_1::submodel_elements::reference_element::ReferenceElement;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display)]
pub enum DataElement {
    Blob(Blob),
    File(File),
    MultiLanguageProperty(MultiLanguageProperty),
    Property(Property),
    Range(Range),
    ReferenceElement(ReferenceElement),
}

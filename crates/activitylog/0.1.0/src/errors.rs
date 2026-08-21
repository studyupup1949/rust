use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct SubjectNotFound(pub String);

impl Display for SubjectNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "Subject not found : '{}'", self.0)
    }
}

impl Error for SubjectNotFound {}

#[derive(Debug)]
pub enum RandomDateElementOutRange {
    Dates(String, String),
    Times(String, String),
}

impl Display for RandomDateElementOutRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomDateElementOutRange::Dates(
                start,
                end
            ) => write!(f, "Random date out of range, choosen between {start} and {end}"),
            RandomDateElementOutRange::Times(
                start,
                end
            ) => write!(f, "Random time out of range, choosen between {start} and {end}"),
        }
    }
}

impl Error for RandomDateElementOutRange {}

#[derive(Debug)]
pub struct PickingEmptySamplesCollection;

impl Display for PickingEmptySamplesCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not pick a sample for a new history record, the samples collection is empty.\nPlease fill the collection with new values.")
    }
}

impl Error for PickingEmptySamplesCollection {}
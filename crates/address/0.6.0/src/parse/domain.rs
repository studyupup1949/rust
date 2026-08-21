use std::convert::TryFrom;
use std::str::FromStr;

use crate::Domain;

impl FromStr for Domain {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

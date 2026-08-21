use enum_map::{Enum, EnumMap};
use serde::Serialize;
use strum_macros::{Display, EnumIter, EnumString};

/// A `Component` is a field of a [`Place`](struct.Place.html)
#[derive(Enum, EnumString, Debug, Clone, EnumIter, Copy, Hash, Display, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum Component {
    /// Leftover field. Can hold a name of a POI, name of building, ... will often be display first
    Attention,
    /// house_number of the place
    HouseNumber,
    /// house of the place
    House,
    /// road of the place
    Road,
    /// village of the place
    Village,
    /// suburb of the place
    Suburb,
    /// city of the place
    City,
    /// county of the place
    County,
    /// county_code of the place
    CountyCode,
    /// postcode of the place
    Postcode,
    /// state_district of the place
    StateDistrict,
    /// state of the place
    State,
    /// state_code of the place
    StateCode,
    /// region of the place
    Region,
    /// island of the place
    Island,
    /// neighbourhood of the place
    Neighbourhood,
    /// country of the place
    Country,
    /// country_code of the place
    CountryCode,
    /// continent of the place
    Continent,
    /// town of the place
    Town,
    /// city_district of the place
    CityDistrict,
}

impl serde::Serialize for Component {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

/// A [`Place`](struct.Place.html) is a structured way to represent a postal address.
///
///
/// Note: it is internally represented as an EnumMap to easily loop over all the fields
#[derive(Debug, Default, Serialize)]
pub struct Place(EnumMap<Component, Option<String>>);

impl std::ops::Deref for Place {
    type Target = EnumMap<Component, Option<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Place {
    fn deref_mut(&mut self) -> &mut EnumMap<Component, Option<String>> {
        &mut self.0
    }
}

impl<'a, T> From<T> for Place
where
    T: IntoIterator<Item = (Component, &'a str)>,
{
    fn from(data: T) -> Self {
        let mut a = Self::default();
        for (k, v) in data.into_iter() {
            a[k] = Some(v.to_owned());
        }
        a
    }
}

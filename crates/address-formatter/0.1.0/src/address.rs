use enum_map::{Enum, EnumMap};
use serde::Serialize;
use strum_macros::{Display, EnumIter, EnumString};

/// A `Component` is a field of an [`Address`](struct.Address.html)
#[derive(Enum, EnumString, Debug, Clone, EnumIter, Copy, Hash, Display, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum Component {
    /// Leftover field. Can hold a name of a POI, name of building, ... will often be display first
    Attention,
    /// house_number of the address
    HouseNumber,
    /// house of the address
    House,
    /// road of the address
    Road,
    /// village of the address
    Village,
    /// suburb of the address
    Suburb,
    /// city of the address
    City,
    /// county of the address
    County,
    /// county_code of the address
    CountyCode,
    /// postcode of the address
    Postcode,
    /// state_district of the address
    StateDistrict,
    /// state of the address
    State,
    /// state_code of the address
    StateCode,
    /// region of the address
    Region,
    /// island of the address
    Island,
    /// neighbourhood of the address
    Neighbourhood,
    /// country of the address
    Country,
    /// country_code of the address
    CountryCode,
    /// continent of the address
    Continent,
    /// town of the address
    Town,
    /// city_district of the address
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

/// An [`Address`](struct.Address.html) is a structured way to represent a postal address.Address
///
///
/// Note: it is internally represented as an EnumMap to easily loop over all the fields
#[derive(Debug, Default, Serialize)]
pub struct Address(EnumMap<Component, Option<String>>);

impl std::ops::Deref for Address {
    type Target = EnumMap<Component, Option<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Address {
    fn deref_mut(&mut self) -> &mut EnumMap<Component, Option<String>> {
        &mut self.0
    }
}

impl<'a, T> From<T> for Address
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

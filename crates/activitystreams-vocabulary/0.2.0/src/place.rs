use crate::{Error, Result, create_object, derived_kind_serde, field_access, impl_into_object};

mod accuracy;
mod float;
mod radius;
mod unit;

pub use accuracy::Accuracy;
pub use float::Float;
pub use radius::Radius;
pub use unit::{Unit, Units};

pub(crate) fn validate_f64(val: f64) -> Result<f64> {
    if val.is_nan() || val.is_infinite() {
        Err(Error::place(format!("invalid float: {val}")))
    } else {
        Ok(val)
    }
}

create_object! {
    /// Represents a logical or physical location.
    ///
    /// # Example (standard units)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Float, Name, Place, Radius, Unit};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Fresno Area").unwrap();
    /// let latitude = Float::from_f64(36.75).unwrap();
    /// let longitude = Float::from_f64(119.7667).unwrap();
    /// let radius = Radius::from_f64(15f64).unwrap();
    /// let units = Unit::Miles;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Place",
    ///   "name": "{name}",
    ///   "latitude": {latitude},
    ///   "longitude": {longitude},
    ///   "radius": {radius},
    ///   "units": "{units}"
    /// }}"#
    ///     );
    ///
    /// let place = Place::new()
    ///     .with_latitude(latitude)
    ///     .with_longitude(longitude)
    ///     .with_radius(radius)
    ///     .with_units(units)
    ///     .with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&place).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Place>(json_str.as_str()).unwrap(),
    ///     place
    /// );
    /// # }
    /// ```
    ///
    /// # Example (unit IRI)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Float, Iri, Name, Place, Radius};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Fresno Area").unwrap();
    /// let latitude = Float::from_f64(36.75).unwrap();
    /// let longitude = Float::from_f64(119.7667).unwrap();
    /// let radius = Radius::from_f64(15f64).unwrap();
    /// let units = Iri::try_from("http://example.org/ns/schema:CustomUnit").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Place",
    ///   "name": "{name}",
    ///   "latitude": {latitude},
    ///   "longitude": {longitude},
    ///   "radius": {radius},
    ///   "units": "{units}"
    /// }}"#
    ///     );
    ///
    /// let place = Place::new()
    ///     .with_latitude(latitude)
    ///     .with_longitude(longitude)
    ///     .with_radius(radius)
    ///     .with_units(units)
    ///     .with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&place).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Place>(json_str.as_str()).unwrap(),
    ///     place
    /// );
    /// # }
    /// ```
    Place:
        #[serde(serialize_with = "obj_serde::ser")]
        ObjectType {
            #[serde(skip_serializing_if = "Option::is_none")]
            latitude: Option<Float>,
            #[serde(skip_serializing_if = "Option::is_none")]
            longitude: Option<Float>,
            #[serde(skip_serializing_if = "Option::is_none")]
            altitude: Option<Float>,
            #[serde(skip_serializing_if = "Option::is_none")]
            accuracy: Option<Accuracy>,
            #[serde(skip_serializing_if = "Option::is_none")]
            radius: Option<Radius>,
            #[serde(skip_serializing_if = "Option::is_none")]
            units: Option<Units>,
        }
}

derived_kind_serde!(crate::ObjectType::Place);
impl_into_object!(Place);

field_access! {
    Place {
        /// The latitude of a place.
        latitude: option { Float },
        /// The longitude of a place.
        longitude: option { Float },
        /// The altitude of a place.
        altitude: option { Float },
        /// Indicates the accuracy of position coordinates on a [Place] objects.
        ///
        /// Expressed in properties of percentage. e.g. "94.0" means "94.0% accurate".
        ///
        /// Valid range is `0.0 ..= 100.0`.
        accuracy: option { Accuracy },
        /// Represents the radius from the given latitude and longitude for a [Place].
        ///
        /// The units is expressed by the `units` property.
        ///
        /// If `units` is not specified, the default is assumed to be "m" indicating "meters".
        radius: option { Radius },
    }
}

field_access! {
    Place {
        /// Specifies the measurement units for the `radius` and `altitude` properties on a [Place] object.
        ///
        /// If not specified, the default is assumed to be "m" for "meters".
        units: option_ref { Units },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name};

    #[test]
    fn test_valid() {
        let name = Name::try_from("Fresno Area").unwrap();
        let latitude = Float::from_f64(36.75).unwrap();
        let longitude = Float::from_f64(119.7667).unwrap();
        let radius = Radius::from_f64(15f64).unwrap();
        let units = Unit::Miles;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Place",
  "name": "{name}",
  "latitude": {latitude},
  "longitude": {longitude},
  "radius": {radius},
  "units": "{units}"
}}"#
        );

        let place = Place::new()
            .with_latitude(latitude)
            .with_longitude(longitude)
            .with_radius(radius)
            .with_units(units)
            .with_name(name);

        assert_eq!(serde_json::to_string_pretty(&place).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Place>(json_str.as_str()).unwrap(),
            place
        );
    }

    #[test]
    fn test_valid_unit_iri() {
        let name = Name::try_from("Fresno Area").unwrap();
        let latitude = Float::from_f64(36.75).unwrap();
        let longitude = Float::from_f64(119.7667).unwrap();
        let radius = Radius::from_f64(15f64).unwrap();
        let units = Iri::try_from("http://example.org/ns/schema:CustomUnit").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Place",
  "name": "{name}",
  "latitude": {latitude},
  "longitude": {longitude},
  "radius": {radius},
  "units": "{units}"
}}"#
        );

        let place = Place::new()
            .with_latitude(latitude)
            .with_longitude(longitude)
            .with_radius(radius)
            .with_units(units)
            .with_name(name);

        assert_eq!(serde_json::to_string_pretty(&place).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Place>(json_str.as_str()).unwrap(),
            place
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Place>(json_str).is_err());
    }
}

///! This module provides convenience access to maidenhead
///! Author: Andreas, DF1ASC@darc.de
use crate::error::Error;
use maidenhead::{grid_bearing, grid_distance, grid_to_longlat, longlat_to_grid};
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops::Sub;
use std::sync::LazyLock;

static RE_GRIDSQUARE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-rA-R]{2}[0-9]{2}([a-xA-X]{2}([0-9]{2}([a-xA-X]{2})?)?)?$").unwrap()
});

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_impl", derive(serde::Serialize, serde::Deserialize))]
/// A coordinate with latitude and longitude.
pub struct Coordinate(f64, f64);

impl Coordinate {
    pub fn new(lat: f64, long: f64) -> Result<Self, Error> {
        if lat > 90.0 || lat < -90.0 {
            return Err(Error::InvalidCoordinate(lat, long));
        }
        if long > 180.0 || long < -180.0 {
            return Err(Error::InvalidCoordinate(lat, long));
        }
        Ok(Self(lat, long))
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Latitude: {:.6}°, Longitude: {:.6}°", self.0, self.1)
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_impl", derive(serde::Serialize, serde::Deserialize))]
/// A GridSquare representation.
pub struct GridSquare(String);

impl GridSquare {
    /// Create a `GridSquare` from string.
    pub fn new(grid: impl AsRef<str>) -> Result<Self, Error> {
        if Regex::is_match(&RE_GRIDSQUARE, grid.as_ref()) {
            let mut grid_str = String::new();
            grid.as_ref()
                .chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .enumerate()
                .for_each(|(i, ch)| {
                    let part = String::from_iter(ch.iter());
                    match i {
                        0 => grid_str.push_str(&part.to_uppercase()),
                        2 | 4 => grid_str.push_str(&part.to_lowercase()),
                        _ => grid_str.push_str(&part),
                    }
                });
            Ok(Self(grid_str))
        } else {
            Err(Error::InvalidGridsquare(grid.as_ref().to_string()))
        }
    }

    /// Creates a `GridSquare` from latitude/longitude with a precision of 4, 6, 8 or 10 chars.
    /// Any other precision value defaults to 6.
    pub fn from_coordinate(coordinate: Coordinate, precision: u8) -> Result<Self, Error> {
        let precision = match precision {
            4 | 8 | 10 => precision,
            _ => 6,
        };
        Ok(Self(
            longlat_to_grid(coordinate.1, coordinate.0, precision as usize)
                .map_err(|_| Error::InvalidCoordinate(coordinate.0, coordinate.1))?,
        ))
    }
}

impl Display for GridSquare {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Coordinate> for GridSquare {
    /// Creates a `GridSquare` from `Coordinate` with precision 6.
    fn from(value: Coordinate) -> Self {
        // Should not fail, due to coordinate is already checked
        Self::from_coordinate(value, 6).unwrap()
    }
}

impl From<GridSquare> for Coordinate {
    fn from(value: GridSquare) -> Self {
        // Should not fail with valid grid
        let longlat = grid_to_longlat(&value.to_string()).unwrap();
        // swap order to common convention
        Coordinate(longlat.1, longlat.0)
    }
}

/// A `Vector` of distance and heading.
#[derive(Debug, PartialEq)]
pub struct Vector(f64, f64);

impl Display for Vector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Distance: {:.2} km, Heading: {:.1}°", self.0, self.1)
    }
}

impl Sub<GridSquare> for GridSquare {
    type Output = Vector;

    fn sub(self, rhs: GridSquare) -> Self::Output {
        // Should not fail, due to valid grid
        Vector(
            grid_distance(&self.to_string(), &rhs.to_string()).unwrap(),
            grid_bearing(&self.to_string(), &rhs.to_string()).unwrap(),
        )
    }
}

impl Sub<Coordinate> for Coordinate {
    type Output = Vector;

    fn sub(self, rhs: Coordinate) -> Self::Output {
        let lhs: GridSquare = self.try_into().unwrap();
        lhs - rhs.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_100_grid() {
        assert_eq!(
            "JO30ui".to_string(),
            GridSquare::new("jo30ui").unwrap().to_string()
        );
        assert_eq!(
            "JO30ui".to_string(),
            GridSquare::new("jo30UI").unwrap().to_string()
        );
        assert_eq!(
            "JO30".to_string(),
            GridSquare::new("jo30").unwrap().to_string()
        );
        assert_eq!(
            "JO30ui44".to_string(),
            GridSquare::new("jo30ui44").unwrap().to_string()
        );

        assert_eq!(GridSquare::new("jo30u").is_err(), true);
        assert_eq!(GridSquare::new("j03ou").is_err(), true);
        assert_eq!(GridSquare::new("jo30ue12er12").is_err(), true);
    }

    #[test]
    fn test_110_grid_coord() {
        assert_eq!(
            Coordinate(50.354166666666686, 7.708333333333314),
            GridSquare::new("jo30ui").unwrap().try_into().unwrap()
        );

        // From lat/long with precision
        assert_eq!(
            GridSquare::new("jo30").unwrap(),
            GridSquare::from_coordinate(Coordinate(50.354166666666686, 7.708333333333314), 4)
                .unwrap()
        );
        assert_eq!(
            GridSquare::new("jo30UI45").unwrap(),
            GridSquare::from_coordinate(Coordinate(50.354166666666686, 7.708333333333314), 8)
                .unwrap()
        );

        // Defaults to precision 6
        assert_eq!(
            GridSquare::new("jo30ui").unwrap(),
            GridSquare::from_coordinate(Coordinate(50.354166666666686, 7.708333333333314), 44)
                .unwrap()
        );
    }

    #[test]
    fn test_200_coordinate() {
        assert_eq!(
            Coordinate::new(-90.1, 7.31),
            Err(Error::InvalidCoordinate(-90.1, 7.31))
        );
        assert_eq!(Coordinate::new(95.0, 7.31).is_err(), true);
        assert_eq!(
            Coordinate::new(50.35, -180.1),
            Err(Error::InvalidCoordinate(50.35, -180.1))
        );
        assert_eq!(Coordinate::new(50.35, 190.0).is_err(), true);
    }

    #[test]
    fn test_210_coordinate_disp() {
        assert_eq!(
            Coordinate::new(50.35, 7.31).unwrap().to_string(),
            "Latitude: 50.350000°, Longitude: 7.310000°"
        );
    }

    #[test]
    fn test_300_vector() {
        let vec = GridSquare::new("JO30uj").unwrap() - GridSquare::new("JO30ui").unwrap();
        assert_eq!(format!("{:.1}", vec.0), "4.6");
        assert_eq!(format!("{:.1}", vec.1), "180.0");

        let vec = GridSquare::new("JO30uj").unwrap() - GridSquare::new("JO30vj").unwrap();
        assert_eq!(format!("{:.1}", vec.0), "5.9");
        assert_eq!(format!("{:.1}", vec.1), "90.0");
    }

    #[test]
    fn test_310_vector_disp() {
        let vec = GridSquare::new("JO30uj").unwrap() - GridSquare::new("JO30ui").unwrap();
        assert_eq!(vec.to_string(), "Distance: 4.63 km, Heading: 180.0°");
    }
}

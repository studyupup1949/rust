use crate::astrolabe::unit::Angle;

/// The latitude and longitude associated with a location.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Coordinates {
    /// Latitude, in degrees
    pub latitude: f64,
    /// Longitude, in degrees
    pub longitude: f64,
}

impl Coordinates {
    /// Latitude as angle
    pub fn latitude_angle(&self) -> Angle {
        Angle::from_degrees(self.latitude)
    }

    /// Longitude as angle
    pub fn longitude_angle(&self) -> Angle {
        Angle::from_degrees(self.longitude)
    }
}

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}{}, {}{})",
            self.latitude.abs(),
            if self.latitude < 0. { 'S' } else { 'N' },
            self.longitude.abs(),
            if self.longitude < 0. { 'W' } else { 'E' },
        )
    }
}

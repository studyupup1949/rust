use crate::{Coordinates, astrolabe::unit::Angle};

/// Calculate Qiblah direction
pub fn qiblah_direction(location_coordinates: Coordinates) -> f64 {
    // Equation from "Spherical Trigonometry For the use of colleges and schools" page 50
    let makkah_coordinates = Coordinates {
        latitude: 21.4225241,
        longitude: 39.8261818,
    };
    let term1 = (makkah_coordinates.longitude_angle().radians()
        - location_coordinates.longitude_angle().radians())
    .sin();
    let term2 = makkah_coordinates.latitude_angle().radians().tan()
        * location_coordinates.latitude_angle().radians().cos();
    let term3 = (makkah_coordinates.longitude_angle().radians()
        - location_coordinates.longitude_angle().radians())
    .cos()
        * location_coordinates.latitude_angle().radians().sin();
    let term4 = term1.atan2(term2 - term3);

    Angle::from_radians(term4).unwound().degrees
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn qiblah_direction_from_nyc_in_north_america() {
        let nyc = {
            let longitude = -74.0059;
            Coordinates {
                latitude: 40.7128,
                longitude,
            }
        };
        let qiblah = qiblah_direction(nyc);

        assert_abs_diff_eq!(qiblah, 58.4817635, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_sf_in_north_america() {
        let sf = {
            let longitude = -122.4194;
            Coordinates {
                latitude: 37.7749,
                longitude,
            }
        };
        let qiblah = qiblah_direction(sf);

        assert_abs_diff_eq!(qiblah, 18.843822245692426, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_dc_in_north_america() {
        let dc = {
            let longitude = -77.0369;
            Coordinates {
                latitude: 38.9072,
                longitude,
            }
        };
        let qiblah = qiblah_direction(dc);

        assert_abs_diff_eq!(qiblah, 56.56046821463599, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_anchorage_in_north_america() {
        let dc = {
            let longitude = -149.9003;
            Coordinates {
                latitude: 61.2181,
                longitude,
            }
        };
        let qiblah = qiblah_direction(dc);

        assert_abs_diff_eq!(qiblah, 350.8830761159853, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_directioon_from_sydney_australia() {
        let sydney = {
            let latitude = -33.8688;
            Coordinates {
                latitude,
                longitude: 151.2093,
            }
        };
        let qiblah = qiblah_direction(sydney);

        assert_abs_diff_eq!(qiblah, 277.4996044487399, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_directioon_from_auckland_new_zealand() {
        let auckland = {
            let latitude = -36.8485;
            Coordinates {
                latitude,
                longitude: 174.7633,
            }
        };
        let qiblah = qiblah_direction(auckland);

        assert_abs_diff_eq!(qiblah, 261.19732640365845, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_london_united_kingdom() {
        let london = {
            let longitude = -0.1278;
            Coordinates {
                latitude: 51.5074,
                longitude,
            }
        };
        let qiblah = qiblah_direction(london);

        assert_abs_diff_eq!(qiblah, 118.9872189, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_paris_france() {
        let paris = Coordinates {
            latitude: 48.8566,
            longitude: 2.3522,
        };
        let qiblah = qiblah_direction(paris);

        assert_abs_diff_eq!(qiblah, 119.16313542183347, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_oslo_norway() {
        let oslo = Coordinates {
            latitude: 59.9139,
            longitude: 10.7522,
        };
        let qiblah = qiblah_direction(oslo);

        assert_abs_diff_eq!(qiblah, 139.02785605537514, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_islamabad_pakistan() {
        let islamabad = Coordinates {
            latitude: 33.7294,
            longitude: 73.0931,
        };
        let qiblah = qiblah_direction(islamabad);

        assert_abs_diff_eq!(qiblah, 255.8816156785436, epsilon = 1e-6);
    }

    #[test]
    fn qiblah_direction_from_tokyo_japan() {
        let tokyo = Coordinates {
            latitude: 35.6895,
            longitude: 139.6917,
        };
        let qiblah = qiblah_direction(tokyo);

        assert_abs_diff_eq!(qiblah, 293.02072441441163, epsilon = 1e-6);
    }
}

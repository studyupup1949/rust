use super::*;

/// Parameters for calculating prayer times.
#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    /// Calculation method
    pub method: &'static dyn Method,
    /// Fajr angle
    pub fajr_angle: f64,
    /// Isha angle
    pub isha_angle: f64,
    /// Maghrib angle
    pub maghrib_angle: f64,
    /// Isha interval, if applicable
    pub isha_interval: Option<u8>,
    /// High latitude rule
    pub high_latitude_rule: HighLatitudeRule,
    /// Polar circle resolution rule
    pub polar_circle_resolver: PolarCircleResolver,
    /// User adjustments
    pub user_adjustments: TimeAdjustment,
}

impl Parameters {
    /// Makes a new set of parameters for the given calculation method.
    pub fn new(method: &'static dyn Method) -> Self {
        Parameters {
            method,
            fajr_angle: method.fajr_angle(),
            isha_angle: method.isha_angle(),
            maghrib_angle: method.maghrib_angle(),
            isha_interval: method.isha_interval(),
            high_latitude_rule: HighLatitudeRule::MiddleOfTheNight,
            polar_circle_resolver: PolarCircleResolver::NearestCity,
            user_adjustments: TimeAdjustment::default(),
        }
    }

    /// Sets the Fajr angle.
    pub fn with_fajr_angle(self, fajr_angle: f64) -> Self {
        Parameters { fajr_angle, ..self }
    }

    /// Sets the Isha angle.
    pub fn with_isha_angle(self, isha_angle: f64) -> Self {
        Parameters { isha_angle, ..self }
    }

    /// Sets the Maghrib angle.
    pub fn with_maghrib_angle(self, maghrib_angle: f64) -> Self {
        Parameters {
            maghrib_angle,
            ..self
        }
    }

    /// Sets time adjustments.
    pub fn with_adjustments(self, adjustments: TimeAdjustment) -> Self {
        Parameters {
            user_adjustments: adjustments,
            ..self
        }
    }

    /// Sets the rule for high latitude calculations.
    pub fn with_high_latitude_rule(self, rule: HighLatitudeRule) -> Self {
        Parameters {
            high_latitude_rule: rule,
            ..self
        }
    }

    /// Sets the polar circle resolver.
    pub fn with_polar_circle_resolver(self, resolver: PolarCircleResolver) -> Self {
        Parameters {
            polar_circle_resolver: resolver,
            ..self
        }
    }

    /// Returns night portions for Fajr and Isha.
    pub fn night_portions(&self) -> (f64, f64) {
        match self.high_latitude_rule {
            HighLatitudeRule::MiddleOfTheNight => (0.5, 0.5),
            HighLatitudeRule::SeventhOfTheNight => (1. / 7., 1. / 7.),
            HighLatitudeRule::TwilightAngle => (self.fajr_angle / 60.0, self.isha_angle / 60.0),
        }
    }

    /// Returns the total time adjustment for a specific prayer.
    pub fn time_adjustments(&self, prayer: Prayer) -> i64 {
        match prayer {
            Prayer::Fajr => self.user_adjustments.fajr + self.method.adjustments().fajr,
            Prayer::Sunrise => self.user_adjustments.sunrise + self.method.adjustments().sunrise,
            Prayer::Dhuhr => self.user_adjustments.dhuhr + self.method.adjustments().dhuhr,
            Prayer::AsrAwwal | Prayer::AsrThaani => {
                self.user_adjustments.asr + self.method.adjustments().asr
            }
            Prayer::Maghrib => self.user_adjustments.maghrib + self.method.adjustments().maghrib,
            Prayer::Isha => self.user_adjustments.isha + self.method.adjustments().isha,
            _ => 0,
        }
    }
}

impl PartialEq for Parameters {
    fn eq(&self, other: &Parameters) -> bool {
        self.fajr_angle == other.fajr_angle
            && self.isha_angle == other.isha_angle
            && self.isha_interval == other.isha_interval
            && self.high_latitude_rule == other.high_latitude_rule
            && self.polar_circle_resolver == other.polar_circle_resolver
            && self.user_adjustments == other.user_adjustments
            && std::ptr::eq(self.method, other.method)
    }
}

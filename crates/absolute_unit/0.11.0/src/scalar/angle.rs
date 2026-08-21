use crate::{
    ArcSeconds, Degrees, DynamicUnits, Radians, Real, Scalar, Unit, impl_value_type_conversions,
    radians, scalar, supports_absdiffeq, supports_cancellation, supports_quantity_ops,
    supports_scalar_ops, supports_shift_ops, supports_value_type_conversion,
};
use std::{fmt, fmt::Debug, marker::PhantomData};

pub trait AngleUnit: Unit + Copy + Debug + Eq + PartialEq + 'static {
    const RADIANS_IN_UNIT: f64;
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Angle<Unit: AngleUnit> {
    v: Real,
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom_1: PhantomData<Unit>,
}
supports_quantity_ops!(Angle<A>, AngleUnit);
supports_shift_ops!(Angle<A1>, Angle<A2>, AngleUnit);
supports_scalar_ops!(Angle<A>, AngleUnit);
supports_cancellation!(Angle<A1>, Angle<A2>, AngleUnit);
supports_absdiffeq!(Angle<A>, AngleUnit);
supports_value_type_conversion!(Angle<A>, AngleUnit, impl_value_type_conversions);

impl<Unit: AngleUnit> Angle<Unit> {
    pub fn floor(self) -> Self {
        self.v.floor().into()
    }

    pub fn ceil(self) -> Self {
        self.v.ceil().into()
    }

    pub fn round(self) -> Self {
        self.v.round().into()
    }

    pub fn abs(self) -> Self {
        self.v.abs().into()
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        if self.v < min.v {
            min
        } else if self.v > max.v {
            max
        } else {
            self
        }
    }

    // In integer units min<x<=max
    pub fn wrap(self, min: Self, max: Self) -> Self {
        debug_assert!(max.v > min.v);
        let range_size = max.v - min.v;
        let mut out = self;
        while out.v <= min.v {
            out.v += range_size;
        }
        while out.v > max.v {
            out.v -= range_size;
        }
        out
    }

    pub fn sign(&self) -> i8 {
        self.v.0.signum() as i8
    }

    pub fn cos(self) -> Scalar {
        scalar!(radians!(self).f64().cos())
    }

    pub fn sin(self) -> Scalar {
        scalar!(radians!(self).f64().sin())
    }

    pub fn tan(self) -> Scalar {
        scalar!(radians!(self).f64().tan())
    }

    pub fn split_degrees_minutes_seconds(&self) -> (i32, i32, i32) {
        let mut arcsecs = Angle::<ArcSeconds>::from(self).f64() as i64;
        let degrees = Angle::<Degrees>::from(self).f64() as i64;
        arcsecs -= degrees * 3_600;
        let minutes = arcsecs / 60;
        arcsecs -= minutes * 60;
        (degrees as i32, minutes as i32, arcsecs as i32)
    }

    pub fn format_latitude(&self) -> String {
        let mut lat = *self;
        let lat_hemi = if lat.f64() >= 0.0 {
            "N"
        } else {
            lat = -lat;
            "S"
        };
        let (lat_d, lat_m, lat_s) = lat.split_degrees_minutes_seconds();
        format!("{lat_hemi}{lat_d:03}d{lat_m:02}m{lat_s:02}s")
    }

    pub fn format_longitude(&self) -> String {
        let mut lon = *self;
        let lon_hemi = if lon.f64() >= 0.0 {
            "E"
        } else {
            lon = -lon;
            "W"
        };
        let (lon_d, lon_m, lon_s) = lon.split_degrees_minutes_seconds();
        format!("{lon_hemi}{lon_d:03}d{lon_m:02}m{lon_s:02}s")
    }

    pub fn as_dyn(&self) -> DynamicUnits {
        DynamicUnits::new1o0::<Unit>(self.v)
    }
}

// Radians are a weirdo that are used as a scalar in many computations.
impl Angle<Radians> {
    pub fn scalar(&self) -> Scalar {
        scalar!(self.v.0)
    }
}

impl<Unit> fmt::Display for Angle<Unit>
where
    Unit: AngleUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.v.0, f)?;
        write!(f, "{}", Unit::UNIT_SUFFIX)
    }
}

impl<'a, UnitA, UnitB> From<&'a Angle<UnitA>> for Angle<UnitB>
where
    UnitA: AngleUnit,
    UnitB: AngleUnit,
{
    fn from(v: &'a Angle<UnitA>) -> Self {
        Self {
            v: v.v * Real(UnitA::RADIANS_IN_UNIT / UnitB::RADIANS_IN_UNIT),
            phantom_1: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{arcminutes, arcseconds, degrees};
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_basic_angle_math() {
        assert_abs_diff_eq!(degrees!(2) + degrees!(2), degrees!(4));
    }

    #[test]
    fn test_arcminute_arcsecond() {
        let a = degrees!(1);
        assert_abs_diff_eq!(arcminutes!(a).f32(), 60f32);
        assert_abs_diff_eq!(arcseconds!(a).f32(), 60f32 * 60f32);
    }

    #[test]
    fn test_wrapping() {
        assert_eq!(
            degrees!(179),
            degrees!(-181).wrap(degrees!(-180), degrees!(180))
        );
        assert_eq!(
            degrees!(-179),
            degrees!(181).wrap(degrees!(-180), degrees!(180))
        );
        assert_abs_diff_eq!(
            degrees!(-179),
            degrees!(180 + 3_600 + 1).wrap(degrees!(-180), degrees!(180)),
            epsilon = 0.000_000_000_001
        );
        assert_abs_diff_eq!(
            degrees!(179),
            degrees!(-180 - 3_600 - 1).wrap(degrees!(-180), degrees!(180)),
            epsilon = 0.000_000_000_001
        );
    }
}

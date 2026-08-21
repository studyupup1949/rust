use crate::{
    AngleUnit, AngularVelocity, DynamicUnits, Real, Time, TimeUnit, impl_value_type_conversions,
    supports_absdiffeq, supports_cancellation, supports_quantity_ops, supports_scalar_ops,
    supports_shift_ops, supports_value_type_conversion,
};
use std::{fmt, fmt::Debug, marker::PhantomData, ops::Mul};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct AngularAcceleration<UnitAngle: AngleUnit, UnitTime: TimeUnit> {
    v: Real,
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom_1: PhantomData<UnitAngle>,
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom_2: PhantomData<UnitTime>,
}
supports_quantity_ops!(AngularAcceleration<A, B>, AngleUnit, TimeUnit);
supports_shift_ops!(AngularAcceleration<A1, B1>, AngularAcceleration<A2, B2>, AngleUnit, TimeUnit);
supports_scalar_ops!(AngularAcceleration<A, B>, AngleUnit, TimeUnit);
supports_cancellation!(AngularAcceleration<A1, B1>, AngularAcceleration<A2, B2>, AngleUnit, TimeUnit);
supports_absdiffeq!(AngularAcceleration<A, B>, AngleUnit, TimeUnit);
supports_value_type_conversion!(AngularAcceleration<A, B>, AngleUnit, TimeUnit, impl_value_type_conversions);

impl<L, T> fmt::Display for AngularAcceleration<L, T>
where
    L: AngleUnit,
    T: TimeUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.v.0, f)?;
        write!(f, "{}/{}²", L::UNIT_SUFFIX, T::UNIT_SHORT_NAME)
    }
}

impl<'a, LA, TA, LB, TB> From<&'a AngularAcceleration<LA, TA>> for AngularAcceleration<LB, TB>
where
    LA: AngleUnit,
    TA: TimeUnit,
    LB: AngleUnit,
    TB: TimeUnit,
{
    fn from(v: &'a AngularAcceleration<LA, TA>) -> Self {
        let angle_ratio = Real(LA::RADIANS_IN_UNIT / LB::RADIANS_IN_UNIT);
        let time_ratio = Real(TB::SECONDS_IN_UNIT / TA::SECONDS_IN_UNIT);
        Self {
            v: v.v * angle_ratio * time_ratio * time_ratio,
            phantom_1: PhantomData,
            phantom_2: PhantomData,
        }
    }
}

impl<L, T> From<DynamicUnits> for AngularAcceleration<L, T>
where
    L: AngleUnit,
    T: TimeUnit,
{
    fn from(v: DynamicUnits) -> Self {
        let f = v.real();
        v.assert_units_equal(DynamicUnits::new1o2::<L, T, T>(Real::ZERO));
        Self {
            v: f,
            phantom_1: PhantomData,
            phantom_2: PhantomData,
        }
    }
}

impl<L, T> AngularAcceleration<L, T>
where
    L: AngleUnit,
    T: TimeUnit,
{
    pub fn as_dyn(&self) -> DynamicUnits {
        DynamicUnits::new1o2::<L, T, T>(self.v)
    }
}

impl<LA, TA, TB> Mul<Time<TB>> for AngularAcceleration<LA, TA>
where
    LA: AngleUnit,
    TA: TimeUnit,
    TB: TimeUnit,
{
    type Output = AngularVelocity<LA, TA>;

    fn mul(self, other: Time<TB>) -> Self::Output {
        AngularVelocity::<LA, TA>::from(self.v.0 * Time::<TA>::from(&other).f64())
    }
}

#[cfg(test)]
mod test {
    use crate::{degrees_per_second2, radians_per_second, radians_per_second2, seconds};
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_angular_acceleration() {
        let r_p_s2 = radians_per_second2!(100.);
        let d_p_s2 = degrees_per_second2!(r_p_s2);
        println!("{r_p_s2}");
        println!("{d_p_s2}");
        assert_abs_diff_eq!(r_p_s2, radians_per_second2!(d_p_s2));
    }

    #[test]
    fn test_angular_accel_shift() {
        let r_p_s2 = radians_per_second2!(100) + degrees_per_second2!(5_732);
        assert_abs_diff_eq!(r_p_s2, radians_per_second2!(200.042), epsilon = 0.001);
    }

    #[test]
    fn test_angular_accel_convert_velocity() {
        let rps2 = radians_per_second2!(100f32);
        assert_abs_diff_eq!(rps2 * seconds!(10f32), radians_per_second!(1000f32));
    }
}

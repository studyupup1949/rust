/// Must be implemented by all unit types.
pub trait Unit {
    const UNIT_NAME: &'static str;
    const UNIT_SHORT_NAME: &'static str;
    const UNIT_SUFFIX: &'static str;
}

// Unitless
pub(crate) mod scalar;

// Angular
pub(crate) mod arcminutes;
pub(crate) mod arcseconds;
pub(crate) mod degrees;
pub(crate) mod radians;

// Distance
pub(crate) mod astronomical_units;
pub(crate) mod feet;
pub(crate) mod kilometers;
pub(crate) mod meters;
pub(crate) mod miles;
pub(crate) mod nautical_miles;

// Temperature
pub(crate) mod celsius;
pub(crate) mod fahrenheit;
pub(crate) mod kelvin;
pub(crate) mod rankine;

// Mass
pub(crate) mod kilograms;
pub(crate) mod pounds_mass;
pub(crate) mod slugs;

// Time
pub(crate) mod hours;
pub(crate) mod seconds;

// Force
pub(crate) mod newtons;
pub(crate) mod pounds_force;

// Pressure
pub(crate) mod pascals;
pub(crate) mod pounds_square_foot;

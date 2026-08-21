use crate::error::AdicShapeError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Adic number input controls, e.g. the number's `p`, `adic_source`, and value
pub struct AdicNumControls {
    /// Prime
    pub p: u32,
    /// What kind of adic integer input are we using
    pub adic_source: AdicNumSource,
    /// The integer for "adic from integer"
    pub from_int_val: i32,
    /// The numerator for "adic from fraction"
    pub numer: i32,
    /// The denominator for "adic from fraction"
    pub denom: u32,
    /// Which preset are we using
    pub preset_idx: usize,
}

impl Default for AdicNumControls {
    fn default() -> Self {
        AdicNumControls{
            p: 5,
            adic_source: AdicNumSource::FromInteger,
            from_int_val: 0,
            numer: 0,
            denom: 1,
            preset_idx: 0,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Creation type for adic number and shape
pub enum AdicNumSource {
    /// Create adic number from integer
    FromInteger,
    /// Create adic number from rational number
    FromRational,
    /// Choose adic number from a preset list
    Preset,
}

impl std::fmt::Display for AdicNumSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AdicNumSource::FromInteger => write!(f, "iadic"),
            AdicNumSource::FromRational => write!(f, "radic"),
            AdicNumSource::Preset => write!(f, "preset"),
        }
    }
}

impl std::str::FromStr for AdicNumSource {
    type Err = AdicShapeError;
    fn from_str(s: &str) -> Result<Self, AdicShapeError> {
        match s {
            "iadic" => Ok(AdicNumSource::FromInteger),
            "radic" => Ok(AdicNumSource::FromRational),
            "preset" => Ok(AdicNumSource::Preset),
            _ => Err(AdicShapeError::Parse("Adic input parse error".to_string()))
        }
    }
}

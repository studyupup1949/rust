use strum::{VariantArray, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, VariantArray, IntoStaticStr)]
pub enum Kind {
    #[default]
    Able,
    Bool,
    Switch,
    Spoken,
}

impl Kind {
    const ABLE: &'static [&'static str; 2] = &["disable", "enable"];
    const BOOL: &'static [&'static str; 2] = &["false", "true"];
    const SWITCH: &'static [&'static str; 2] = &["off", "on"];
    const SPOKEN: &'static [&'static str; 2] = &["no", "yes"];

    pub fn names(self) -> &'static [&'static str; 2] {
        match self {
            Self::Able => &Self::ABLE,
            Self::Bool => &Self::BOOL,
            Self::Switch => &Self::SWITCH,
            Self::Spoken => &Self::SPOKEN,
        }
    }
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str((*self).into())
    }
}
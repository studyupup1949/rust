use syn::{BoundLifetimes, Path, TraitBound, TraitBoundModifier, token::Paren};

pub trait TraitBoundConstructExt {
    fn from_parts(
        paren_token: Option<Paren>,
        modifier: TraitBoundModifier,
        lifetimes: Option<BoundLifetimes>,
        path: Path,
    ) -> TraitBound;

    fn from_path(path: Path) -> TraitBound;
}

impl TraitBoundConstructExt for TraitBound {
    fn from_parts(
        paren_token: Option<Paren>,
        modifier: TraitBoundModifier,
        lifetimes: Option<BoundLifetimes>,
        path: Path,
    ) -> TraitBound {
        TraitBound {
            paren_token,
            modifier,
            lifetimes,
            path,
        }
    }

    fn from_path(path: Path) -> TraitBound {
        <Self as TraitBoundConstructExt>::from_parts(None, TraitBoundModifier::None, None, path)
    }
}

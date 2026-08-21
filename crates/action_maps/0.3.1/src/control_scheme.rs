use bevy_ecs::system::Resource;
use std::collections::HashMap;

use crate::action::Action;
use crate::input::UniversalInput;

/// A wrapper around a map of `Action`s to `UniversalInput`s.  
/// Example:
/// ```
/// use bevy::prelude::*;
/// use action_maps::prelude::*;
/// use action_maps::get_scan_code;
///
/// fn bind_keys(mut control_scheme: ResMut<ControlScheme>) {
///    control_scheme.insert("Up", ScanCode(get_scan_code("W").unwrap()));
///    control_scheme.insert("Left", KeyCode::A);
///    control_scheme.insert("Shoot", MouseButton::Left);
/// }
/// ```
#[derive(Debug, Clone, Resource, Default, PartialEq, Eq)]
pub struct ControlScheme(HashMap<UniversalInput, Action>);

#[allow(dead_code)]
impl ControlScheme {
    pub fn set(&mut self, other: ControlScheme) {
        self.0 = other.0;
    }

    pub fn insert<A, I>(&mut self, action: A, input: I)
    where
        A: Into<Action>,
        I: Into<UniversalInput>,
    {
        self.0.insert(input.into(), action.into());
    }

    pub fn remove<I>(&mut self, input: I)
    where
        I: Into<UniversalInput>,
    {
        self.0.remove(&input.into());
    }

    pub fn get<I>(&self, input: I) -> Option<&Action>
    where
        I: Into<UniversalInput>,
    {
        self.0.get(&input.into())
    }

    pub fn get_mut<I>(&mut self, input: I) -> Option<&mut Action>
    where
        I: Into<UniversalInput>,
    {
        self.0.get_mut(&input.into())
    }

    pub fn contains_key<I>(&self, input: I) -> bool
    where
        I: Into<UniversalInput>,
    {
        self.0.contains_key(&input.into())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&UniversalInput, &Action)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&UniversalInput, &mut Action)> {
        self.0.iter_mut()
    }
}

/// Eases the creation of large control schemes by accepting any number of tuples with
/// with the type `(A: Into<Action>, I: Into<UniversalInput>)`.
/// ```rust
/// use action_maps::prelude::*;
/// use bevy::prelude::*;
///
/// let mut control_scheme = ControlScheme::default();
/// control_scheme.insert("A", KeyCode::A);
/// control_scheme.insert("W", KeyCode::W);
///
/// let with_macro = make_controls!(
///     ("A", KeyCode::A),
///     ("W", KeyCode::W),
/// );
///
/// assert_eq!(control_scheme, with_macro);
/// ```
#[macro_export]
macro_rules! make_controls {
    ( $( ($A: expr, $I: expr) $(,)? ),*) => {
        {
            let mut controls = ControlScheme::default();
            $(
                controls.insert($A, $I);
            )*
            controls
        }
    }
}

#[test]
fn test_make_controls() {
    use bevy::prelude::KeyCode;

    let mut cs = ControlScheme::default();
    let mut cs_t = ControlScheme::default();

    cs.insert("A", KeyCode::A);
    cs.insert("W", KeyCode::W);

    cs_t.set(make_controls!(("A", KeyCode::A), ("W", KeyCode::W)));

    assert_eq!(cs, cs_t);
}

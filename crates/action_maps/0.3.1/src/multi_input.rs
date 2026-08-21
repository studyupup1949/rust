use crate::action_input::ActionInput;
use bevy_ecs::system::Resource;
use std::collections::HashMap;

/// Helper function to be enable local multiplayer
/// ```rust
/// use action_maps::multiplayer_prelude::*;
/// use bevy::prelude::*;
///
/// #[derive(Component)]
/// struct PlayerId(usize);
///
/// fn setup(mut inputs: ResMut<MultiInput>) {
///     inputs.has_players(2);
/// }
///
/// fn handle_input(
///     multi_input: Res<MultiInput>,
///     mut query: Query<(&mut Transform, &PlayerId)>,
/// ) {
///     for (mut transform, PlayerId(id)) in query.iter_mut() {
///         let actions: &ActionInput = multi_input.get(*id).unwrap();
///         // handle input as normal
///     }
/// }
/// ```
#[derive(Debug, Clone, Resource, Default)]
pub struct MultiInput {
    map: HashMap<usize, ActionInput>,
}

impl PartialEq for MultiInput {
    fn eq(&self, other: &Self) -> bool {
        let mut self_keys = self.keys().collect::<Vec<&usize>>();
        let mut other_keys = other.keys().collect::<Vec<&usize>>();

        self_keys.sort_unstable();
        other_keys.sort_unstable();

        if self_keys.len() != other_keys.len() {
            return false;
        }

        for i in 0..self_keys.len() {
            if self_keys[i] != other_keys[i] {
                return false;
            }
        }

        return true;
    }
}

impl MultiInput {
    pub fn new() -> Self {
        let map = HashMap::new();
        Self { map }
    }

    pub fn get(&self, id: usize) -> Option<&ActionInput> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut ActionInput> {
        self.map.get_mut(&id)
    }

    pub fn insert(&mut self, id: usize) {
        self.map.insert(id, ActionInput::default());
    }

    pub fn has_players(&mut self, count: usize) {
        for i in 0..count {
            self.insert(i);
        }
    }

    pub fn remove(&mut self, id: usize) {
        self.map.remove(&id);
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, usize, ActionInput> {
        self.map.keys()
    }
}

/// Eases the setup process for binding keys for multiplayer. The first argument is a
/// MultiInput object, the second is a MultiScheme object, and the remaining objects
/// are tuples containing one or more tuples of type
/// `(A: Into<Action>, I: Into<UniversalInput>)`.
/// ```rust
/// use bevy::prelude::*;
/// use action_maps::multiplayer_prelude::*;
///
/// fn setup(mut inputs: ResMut<MultiInput>, mut schemes: ResMut<MultiScheme>) {
///     make_multi_input!(
///         inputs,
///         schemes,
///         (
///             ("A", KeyCode::A),
///         ),
///         (
///             ("Left", KeyCode::Left),
///         )
///     )
/// }
/// ```
#[macro_export]
macro_rules! make_multi_input {
    ($multi_input:ident, $multi_scheme:ident, $( ( $( ($A:expr, $I:expr) $(,)? ),* ) ),*    ) => {
        {
            use $crate::make_controls;
            use $crate::controls::ControlScheme;

            let mut __count = 0;
            $(
            let __controls = make_controls!(
            $(($A, $I)),*
            );
            $multi_scheme.insert(__count, __controls);
            __count += 1;
            )*

            $multi_input.has_players(__count);
        }
    }
}

#[test]
fn test_make_multi_input() {
    use crate::controls::ControlScheme;
    use crate::controls::MultiScheme;
    use crate::make_controls;
    use bevy::prelude::KeyCode;

    let mut mi = MultiInput::default();
    let mut mi_t = MultiInput::default();

    let mut ms = MultiScheme::default();
    let mut ms_t = MultiScheme::default();

    ms.insert(0, make_controls!(("A", KeyCode::A), ("W", KeyCode::W),));

    ms.insert(
        1,
        make_controls!(("Up", KeyCode::Up), ("Down", KeyCode::Down),),
    );

    mi.has_players(2);

    make_multi_input!(
        mi_t,
        ms_t,
        (("A", KeyCode::A), ("W", KeyCode::W),),
        (("Up", KeyCode::Up), ("Down", KeyCode::Down),)
    );

    assert_eq!(mi, mi_t);
    assert_eq!(ms, ms_t);
}

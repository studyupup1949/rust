use crate::input::ActionInput;
use bevy_ecs::system::Resource;
use std::collections::HashMap;

/// Helper function to be enable local multiplayer
/// ```rust
/// use action_maps::multiplayer::*;
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

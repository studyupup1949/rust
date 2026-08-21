use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet};

use crate::action_input::ActionInput;
use crate::{
    control_scheme::ControlScheme, multi_input::MultiInput, multi_scheme::MultiScheme,
};

/// All systems handling controls should be a member of the `HandleActions` set
/// to ensure that they run after the UniversalInputPlugin can update the Input
/// resource.
/// ```rust
/// use bevy::prelude::*;
/// use action_maps::prelude::*;
///
/// fn main() {
///    App::new()
///        .add_plugins(ActionMapPlugin)
///        .add_systems(
///            PreUpdate,
///            handle_input.in_set(ActionMapSet::HandleActions),
///        )
///     ;
/// }
///
/// fn handle_input() {}
/// ```

#[derive(Hash, Debug, PartialEq, Eq, Clone, SystemSet)]
pub enum ActionMapSet {
    ReadEvents,
    HandleActions,
}

pub struct ActionMapPlugin;

impl Plugin for ActionMapPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            ActionMapSet::HandleActions.after(ActionMapSet::ReadEvents),
        )
        .init_resource::<ControlScheme>()
        .init_resource::<ActionInput>()
        .init_resource::<bevy_input::gamepad::GamepadSettings>()
        .add_event::<bevy_input::keyboard::KeyboardInput>()
        .add_event::<bevy_input::gamepad::GamepadButtonChangedEvent>()
        .add_event::<bevy_input::gamepad::GamepadButtonInput>()
        .add_event::<bevy_input::mouse::MouseButtonInput>()
        .add_systems(
            PreUpdate,
            (crate::input::universal_input_system).in_set(ActionMapSet::ReadEvents),
        );
    }
}

pub struct MultiActionMapPlugin;

impl Plugin for MultiActionMapPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            ActionMapSet::HandleActions.after(ActionMapSet::ReadEvents),
        )
        .init_resource::<MultiScheme>()
        .init_resource::<MultiInput>()
        .init_resource::<bevy_input::gamepad::GamepadSettings>()
        .add_event::<bevy_input::keyboard::KeyboardInput>()
        .add_event::<bevy_input::gamepad::GamepadButtonChangedEvent>()
        .add_event::<bevy_input::gamepad::GamepadButtonInput>()
        .add_event::<bevy_input::mouse::MouseButtonInput>()
        .add_systems(
            PreUpdate,
            (crate::input::multi_universal_input_system)
                .in_set(ActionMapSet::ReadEvents),
        );
    }
}

use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::{
    change_detection::DetectChangesMut,
    schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet},
    system::{Res, ResMut},
};
use bevy_input::{
    gamepad::GamepadButton,
    keyboard::{KeyCode, ScanCode},
    mouse::MouseButton,
    Input as BevyInput,
};

use crate::input::ActionInput;
use crate::input_type::Key;
use crate::input_type::UniversalInput;
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

unsafe impl Send for Action {}
unsafe impl Sync for Action {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Action {
    pub name: &'static str,
}

impl From<&'static str> for Action {
    fn from(name: &'static str) -> Self {
        Action { name }
    }
}

pub fn action_input_system(
    mut actions: ResMut<ActionInput>,
    keycodes: Res<BevyInput<KeyCode>>,
    scancodes: Res<BevyInput<ScanCode>>,
    mouse_buttons: Res<BevyInput<MouseButton>>,
    gamepad_buttons: Res<BevyInput<GamepadButton>>,
    control_scheme: Res<ControlScheme>,
) {
    actions.bypass_change_detection().clear();
    for (action, input) in control_scheme.iter() {
        match input {
            UniversalInput::Keyboard(key) => match key {
                Key::KeyCode(kc) => {
                    if keycodes.just_pressed(*kc) {
                        actions.press(*action);
                    }
                    if keycodes.just_released(*kc) {
                        actions.release(*action);
                    }
                }
                Key::ScanCode(sc) => {
                    if scancodes.just_pressed(*sc) {
                        actions.press(*action);
                    }
                    if scancodes.just_released(*sc) {
                        actions.release(*action);
                    }
                }
            },
            UniversalInput::MouseButton(mb) => {
                if mouse_buttons.just_pressed(*mb) {
                    actions.press(*action);
                }
                if mouse_buttons.just_released(*mb) {
                    actions.release(*action);
                }
            }
            UniversalInput::GamepadButton(gb) => {
                if gamepad_buttons.just_pressed(*gb) {
                    actions.press(*action);
                }
                if gamepad_buttons.just_released(*gb) {
                    actions.release(*action);
                }
            }
        }
    }
}

pub fn multi_action_input_system(
    mut inputs: ResMut<MultiInput>,
    keycodes: Res<BevyInput<KeyCode>>,
    scancodes: Res<BevyInput<ScanCode>>,
    mouse_buttons: Res<BevyInput<MouseButton>>,
    gamepad_buttons: Res<BevyInput<GamepadButton>>,
    schemes: Res<MultiScheme>,
) {
    let ids: Vec<usize> = inputs.keys().copied().collect();
    for i in ids {
        let actions = inputs.get_mut(i).unwrap();
        let control_scheme = schemes.get(i).unwrap();

        // (*actions).bypass_change_detection().clear();
        for (action, input) in control_scheme.iter() {
            match input {
                UniversalInput::Keyboard(key) => match key {
                    Key::KeyCode(kc) => {
                        if keycodes.just_pressed(*kc) {
                            actions.press(*action);
                        }
                        if keycodes.just_released(*kc) {
                            actions.release(*action);
                        }
                    }
                    Key::ScanCode(sc) => {
                        if scancodes.just_pressed(*sc) {
                            actions.press(*action);
                        }
                        if scancodes.just_released(*sc) {
                            actions.release(*action);
                        }
                    }
                },
                UniversalInput::MouseButton(mb) => {
                    if mouse_buttons.just_pressed(*mb) {
                        actions.press(*action);
                    }
                    if mouse_buttons.just_released(*mb) {
                        actions.release(*action);
                    }
                }
                UniversalInput::GamepadButton(gb) => {
                    if gamepad_buttons.just_pressed(*gb) {
                        actions.press(*action);
                    }
                    if gamepad_buttons.just_released(*gb) {
                        actions.release(*action);
                    }
                }
            }
        }
    }
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
        .add_systems(
            PreUpdate,
            (action_input_system).in_set(ActionMapSet::ReadEvents),
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
        .add_systems(
            PreUpdate,
            (multi_action_input_system).in_set(ActionMapSet::ReadEvents),
        );
    }
}

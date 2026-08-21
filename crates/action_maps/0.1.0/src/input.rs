use crate::action::Action;
use bevy_ecs::system::Resource;
use bevy_input::Input as BevyInput;

/// A wrapper around `bevy_input::Input` for use with the more generic `Action`s.
/// The interface is exactly the same as `bevy_input::Input`. See `ControlScheme` for details on
/// how to bind inputs to actions.
/// ```rust
/// use bevy::prelude::*;
/// use action_maps::prelude::*;
///
/// fn handle_input(mut input: ResMut<ActionInput>) {
///    if input.pressed("Up") {
///        println!("Up is pressed!");
///    }
/// }
/// ```
#[derive(Debug, Clone, Resource, Default)]
pub struct ActionInput(BevyInput<Action>);

impl ActionInput {
    pub fn press<A>(&mut self, input: A)
    where
        A: Into<Action>,
    {
        self.0.press(input.into());
    }

    pub fn pressed<A>(&self, input: A) -> bool
    where
        A: Into<Action>,
    {
        self.0.pressed(input.into())
    }

    pub fn any_pressed<A>(&self, inputs: impl IntoIterator<Item = A>) -> bool
    where
        A: Into<Action>,
    {
        let inputs = inputs.into_iter().map(|a| a.into()).collect::<Vec<_>>();
        self.0.any_pressed(inputs)
    }

    pub fn release<A>(&mut self, input: A)
    where
        A: Into<Action>,
    {
        self.0.release(input.into());
    }

    pub fn release_all(&mut self) {
        self.0.release_all();
    }

    pub fn just_pressed<A>(&self, input: A) -> bool
    where
        A: Into<Action>,
    {
        self.0.just_pressed(input.into())
    }

    pub fn any_just_pressed<A>(
        &self,
        inputs: impl IntoIterator<Item = A>,
    ) -> bool
    where
        A: Into<Action>,
    {
        let inputs = inputs.into_iter().map(|a| a.into()).collect::<Vec<_>>();
        self.0.any_just_pressed(inputs)
    }

    pub fn clear_just_pressed<A>(&mut self, input: A)
    where
        A: Into<Action>,
    {
        self.0.clear_just_pressed(input.into());
    }

    pub fn reset<A>(&mut self, input: A)
    where
        A: Into<Action>,
    {
        self.0.reset(input.into());
    }

    pub fn reset_all(&mut self) {
        self.0.reset_all();
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn get_pressed(&self) -> impl ExactSizeIterator<Item = &Action> {
        self.0.get_pressed()
    }

    pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item = &Action> {
        self.0.get_just_pressed()
    }

    pub fn get_just_released(&self) -> impl ExactSizeIterator<Item = &Action> {
        self.0.get_just_released()
    }
}

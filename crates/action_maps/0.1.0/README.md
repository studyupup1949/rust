# Action Maps

A dynamic action mapping system for Bevy.

## Why Action Maps?

Action maps provides an interface similar to Unity's InputSystem and Godot's
Input Map. With Action Maps, you can assign functionality to any available Bevy
input item without having to work directly with they inputs themselves. 

At the moment, this only applies to button type inputs. Axis type inputs are
planned to be implemented in the future.

## Defining Actions

Under the hood, actions only use a String to keep track of their identity.
Defining actions is as easy as inserting the name of the action and the input
for the action into the control scheme. 

```rust
fn bind_keys(
    mut control_scheme: ResMut<ControlScheme>
) {
    control_scheme.insert("Jump", KeyCode::Space);
}
```

## Handling input

Action maps are wrappers around the existing Bevy `Input<T>`, so the interface
for using them is exactly the same. 

```rust
use action_maps::prelude::*;

fn handle_input(
    actions: Res<ActionInput>
) {
    if action.just_pressed("Jump") {
        println!("Your character jumps!");
    }
}
```

## A Different Way

Action maps methods all accept any item which implements `Into<Action>`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Actions {
    Up,
    Left,
    Down,
    Right
}

impl Into<Action> for Actions {
    fn into(self) -> Action {
        match self {
            Actions::Up => Action::from("Up"),
            Actions::Left => Action::from("Left"),
            Actions::Down => Action::from("Down"),
            Actions::Right => Action::from("Right"),
            Actions::ChangeColor => Action::from("ChangeColor"),
        }
    }
}

fn bind_keys(
    mut control_scheme: ResMut<ControlScheme>
) {
    control_scheme.insert(Actions::Up, KeyCode::W);
}
```

## Universal Input

Action maps provides another layer to help you define keybindings:
`UniversalInput`. UniversalInput is a simple wrapper over every button-type
input device available in Bevy. Action maps methods can take any of these input
types and use them just the same.

```rust
fn bind_keys(
    mut control_scheme: ResMut<ControlScheme>
) {
    control_scheme.insert("Up", KeyCode::W);
    control_scheme.insert("Down", ScanCode(get_scan_code("W")));
    control_scheme.insert("Shoot", MouseButton::Left);
}
```

## Using ScanCodes

Action maps provides a helper function `action_maps::get_scan_code` to
assist in using ScanCodes in Bevy. This function is extremely rudimentary and
likely prone to missing keys (not to mention completely lacking support for
Linux). Hopefully, it will be made unnecessary with
[this](https://github.com/bevyengine/bevy/pull/10702) PR and Bevy 0.13.

```rust
let qwerty_w_scancode = action_maps::get_scan_code("W");
```

## Examples

See `examples/keyboard_and_mouse.rs` for a complete mockup of how action maps
work in practice. 

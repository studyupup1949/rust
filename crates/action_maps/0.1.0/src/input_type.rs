use bevy_input::gamepad::GamepadButton;
// use bevy_input::gamepad::GamepadButtonType;
use bevy_input::keyboard::KeyCode;
use bevy_input::keyboard::ScanCode;
use bevy_input::mouse::MouseButton;

/// Represents a type of input that can be mapped to an action.
/// Allows control schemes to be more generic
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum UniversalInput {
    Keyboard(Key),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Key {
    KeyCode(KeyCode),
    ScanCode(ScanCode),
}

impl From<KeyCode> for UniversalInput {
    fn from(key_code: KeyCode) -> Self {
        UniversalInput::Keyboard(Key::KeyCode(key_code))
    }
}

impl From<ScanCode> for UniversalInput {
    fn from(scan_code: ScanCode) -> Self {
        UniversalInput::Keyboard(Key::ScanCode(scan_code))
    }
}

impl From<GamepadButton> for UniversalInput {
    fn from(gamepad_button: GamepadButton) -> Self {
        UniversalInput::GamepadButton(gamepad_button)
    }
}

impl From<MouseButton> for UniversalInput {
    fn from(mouse_button: MouseButton) -> Self {
        UniversalInput::MouseButton(mouse_button)
    }
}

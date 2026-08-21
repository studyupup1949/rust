use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::event::Event;
use bevy_ecs::event::EventReader;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use bevy_input::gamepad::GamepadButton;
use bevy_input::gamepad::GamepadButtonChangedEvent;
use bevy_input::gamepad::GamepadButtonInput;
use bevy_input::gamepad::GamepadButtonType;
use bevy_input::gamepad::GamepadSettings;
use bevy_input::keyboard::KeyCode;
use bevy_input::keyboard::KeyboardInput;
use bevy_input::keyboard::ScanCode;
use bevy_input::mouse::MouseButton;
use bevy_input::mouse::MouseButtonInput;
use bevy_input::ButtonState;
use bevy_log::warn;
use bevy_reflect::Enum;

use crate::actions::MultiInput;
use crate::controls::MultiScheme;
use crate::get_key;
use crate::get_scan_code;
use crate::prelude::ActionInput;
use crate::prelude::ControlScheme;

pub fn universal_input_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut gamepad_events: EventReader<GamepadButtonChangedEvent>,
    mut button_input_events: EventWriter<GamepadButtonInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut action_input: ResMut<ActionInput>,
    control_scheme: Res<ControlScheme>,
    settings: Res<GamepadSettings>,
) {
    action_input.bypass_change_detection().clear();
    let keyboard_events: Vec<&KeyboardInput> = keyboard_events.read().collect();
    let gamepad_events: Vec<&GamepadButtonChangedEvent> =
        gamepad_events.read().collect();
    let mouse_button_events: Vec<&MouseButtonInput> =
        mouse_button_events.read().collect();
    update_inputs(
        &keyboard_events,
        &gamepad_events,
        &mut button_input_events,
        &mouse_button_events,
        &mut action_input,
        &control_scheme,
        &settings,
    );
}

pub fn multi_universal_input_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut gamepad_events: EventReader<GamepadButtonChangedEvent>,
    mut button_input_writer: EventWriter<GamepadButtonInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut multi_input: ResMut<MultiInput>,
    multi_scheme: Res<MultiScheme>,
    settings: Res<GamepadSettings>,
) {
    multi_input.bypass_change_detection();
    let keyboard_events: Vec<&KeyboardInput> = keyboard_events.read().collect();
    let gamepad_events: Vec<&GamepadButtonChangedEvent> =
        gamepad_events.read().collect();
    let mouse_button_events: Vec<&MouseButtonInput> =
        mouse_button_events.read().collect();

    for i in 0..multi_input.keys().len() {
        let action_input = multi_input.get_mut(i).unwrap();
        let control_scheme = multi_scheme.get(i).unwrap();
        action_input.clear();
        update_inputs(
            &keyboard_events,
            &gamepad_events,
            &mut button_input_writer,
            &mouse_button_events,
            action_input,
            control_scheme,
            &settings,
        );
    }
}

fn update_inputs(
    keyboard_events: &Vec<&KeyboardInput>,
    gamepad_events: &Vec<&GamepadButtonChangedEvent>,
    button_input_events: &mut EventWriter<GamepadButtonInput>,
    mouse_button_events: &Vec<&MouseButtonInput>,
    action_input: &mut ActionInput,
    control_scheme: &ControlScheme,
    settings: &Res<GamepadSettings>,
) {
    for event in keyboard_events {
        let KeyboardInput {
            scan_code, state, ..
        } = event;

        let key: UniversalInput = ScanCode(*scan_code).into();
        if let Some(action) = control_scheme.get(key) {
            match state {
                ButtonState::Pressed => action_input.press(*action),
                ButtonState::Released => action_input.release(*action),
            }
        }
    }

    for event in gamepad_events {
        let button = GamepadButton::new(event.gamepad, event.button_type);
        let value = event.value;
        let button_settings = settings.get_button_settings(button);
        let input: UniversalInput = button.into();
        let Some(action) = control_scheme.get(input) else {
            return;
        };

        // if is released...
        if value <= button_settings.release_threshold() {
            if action_input.pressed(*action) {
                button_input_events.send(GamepadButtonInput {
                    button,
                    state: ButtonState::Released,
                });
            }
            action_input.release(*action);
        } else if value >= button_settings.press_threshold() {
            button_input_events.send(GamepadButtonInput {
                button,
                state: ButtonState::Pressed,
            });
            action_input.press(*action);
        }
    }

    for event in mouse_button_events {
        let button: UniversalInput = event.button.into();

        if let Some(action) = control_scheme.get(button) {
            match event.state {
                ButtonState::Pressed => action_input.press(*action),
                ButtonState::Released => action_input.release(*action),
            }
        }
    }
}

#[derive(Event)]
pub struct UniversalInputEvent(UniversalInput);

/// Keys represent the physical key
#[repr(u32)]
#[derive(Hash, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum UniversalInput {
    /// The `1` key over the letters.
    Key1,
    /// The `2` key over the letters.
    Key2,
    /// The `3` key over the letters.
    Key3,
    /// The `4` key over the letters.
    Key4,
    /// The `5` key over the letters.
    Key5,
    /// The `6` key over the letters.
    Key6,
    /// The `7` key over the letters.
    Key7,
    /// The `8` key over the letters.
    Key8,
    /// The `9` key over the letters.
    Key9,
    /// The `0` key over the letters.
    Key0,

    /// The `A` key.
    A,
    /// The `B` key.
    B,
    /// The `C` key.
    C,
    /// The `D` key.
    D,
    /// The `E` key.
    E,
    /// The `F` key.
    F,
    /// The `G` key.
    G,
    /// The `H` key.
    H,
    /// The `I` key.
    I,
    /// The `J` key.
    J,
    /// The `K` key.
    K,
    /// The `L` key.
    L,
    /// The `M` key.
    M,
    /// The `N` key.
    N,
    /// The `O` key.
    O,
    /// The `P` key.
    P,
    /// The `Q` key.
    Q,
    /// The `R` key.
    R,
    /// The `S` key.
    S,
    /// The `T` key.
    T,
    /// The `U` key.
    U,
    /// The `V` key.
    V,
    /// The `W` key.
    W,
    /// The `X` key.
    X,
    /// The `Y` key.
    Y,
    /// The `Z` key.
    Z,

    /// The `Escape` / `ESC` key, next to the `F1` key.
    Escape,

    /// The `F1` key.
    F1,
    /// The `F2` key.
    F2,
    /// The `F3` key.
    F3,
    /// The `F4` key.
    F4,
    /// The `F5` key.
    F5,
    /// The `F6` key.
    F6,
    /// The `F7` key.
    F7,
    /// The `F8` key.
    F8,
    /// The `F9` key.
    F9,
    /// The `F10` key.
    F10,
    /// The `F11` key.
    F11,
    /// The `F12` key.
    F12,
    /// The `F13` key.
    F13,
    /// The `F14` key.
    F14,
    /// The `F15` key.
    F15,
    /// The `F16` key.
    F16,
    /// The `F17` key.
    F17,
    /// The `F18` key.
    F18,
    /// The `F19` key.
    F19,
    /// The `F20` key.
    F20,
    /// The `F21` key.
    F21,
    /// The `F22` key.
    F22,
    /// The `F23` key.
    F23,
    /// The `F24` key.
    F24,

    /// The `Snapshot` / `Print Screen` key.
    Snapshot,
    /// The `Scroll` / `Scroll Lock` key.
    Scroll,
    /// The `Pause` / `Break` key, next to the `Scroll` key.
    Pause,

    /// The `Insert` key, next to the `Backspace` key.
    Insert,
    /// The `Home` key.
    Home,
    /// The `Delete` key.
    Delete,
    /// The `End` key.
    End,
    /// The `PageDown` key.
    PageDown,
    /// The `PageUp` key.
    PageUp,

    /// The `Left` / `Left Arrow` key.
    Left,
    /// The `Up` / `Up Arrow` key.
    Up,
    /// The `Right` / `Right Arrow` key.
    Right,
    /// The `Down` / `Down Arrow` key.
    Down,

    /// The `Back` / `Backspace` key.
    Back,
    /// The `Return` / `Enter` key.
    Return,
    /// The `Space` / `Spacebar` / ` ` key.
    Space,

    /// The `Compose` key on Linux.
    Compose,
    /// The `Caret` / `^` key.
    Caret,

    /// The `Numlock` key.
    Numlock,
    /// The `Numpad0` / `0` key.
    Numpad0,
    /// The `Numpad1` / `1` key.
    Numpad1,
    /// The `Numpad2` / `2` key.
    Numpad2,
    /// The `Numpad3` / `3` key.
    Numpad3,
    /// The `Numpad4` / `4` key.
    Numpad4,
    /// The `Numpad5` / `5` key.
    Numpad5,
    /// The `Numpad6` / `6` key.
    Numpad6,
    /// The `Numpad7` / `7` key.
    Numpad7,
    /// The `Numpad8` / `8` key.
    Numpad8,
    /// The `Numpad9` / `9` key.
    Numpad9,

    /// The `AbntC1` key.
    AbntC1,
    /// The `AbntC2` key.
    AbntC2,

    /// The `NumpadAdd` / `+` key.
    NumpadAdd,
    /// The `Apostrophe` / `'` key.
    Apostrophe,
    /// The `Apps` key.
    Apps,
    /// The `Asterisk` / `*` key.
    Asterisk,
    /// The `Plus` / `+` key.
    Plus,
    /// The `At` / `@` key.
    At,
    /// The `Ax` key.
    Ax,
    /// The `Backslash` / `\` key.
    Backslash,
    /// The `Calculator` key.
    Calculator,
    /// The `Capital` key.
    Capital,
    /// The `Colon` / `:` key.
    Colon,
    /// The `Comma` / `,` key.
    Comma,
    /// The `Convert` key.
    Convert,
    /// The `NumpadDecimal` / `.` key.
    NumpadDecimal,
    /// The `NumpadDivide` / `/` key.
    NumpadDivide,
    /// The `Equals` / `=` key.
    Equals,
    /// The `Grave` / `Backtick` / `` ` `` key.
    Grave,
    /// The `Kana` key.
    Kana,
    /// The `Kanji` key.
    Kanji,

    /// The `Left Alt` key. Maps to `Left Option` on Mac.
    AltLeft,
    /// The `Left Bracket` / `[` key.
    BracketLeft,
    /// The `Left Control` key.
    ControlLeft,
    /// The `Left Shift` key.
    ShiftLeft,
    /// The `Left Super` key.
    /// Generic keyboards usually display this key with the *Microsoft Windows* logo.
    /// Apple keyboards call this key the *Command Key* and display it using the ⌘ character.
    #[doc(alias("LWin", "LMeta", "LLogo"))]
    SuperLeft,

    /// The `Mail` key.
    Mail,
    /// The `MediaSelect` key.
    MediaSelect,
    /// The `MediaStop` key.
    MediaStop,
    /// The `Minus` / `-` key.
    Minus,
    /// The `NumpadMultiply` / `*` key.
    NumpadMultiply,
    /// The `Mute` key.
    Mute,
    /// The `MyComputer` key.
    MyComputer,
    /// The `NavigateForward` / `Prior` key.
    NavigateForward,
    /// The `NavigateBackward` / `Next` key.
    NavigateBackward,
    /// The `NextTrack` key.
    NextTrack,
    /// The `NoConvert` key.
    NoConvert,
    /// The `NumpadComma` / `,` key.
    NumpadComma,
    /// The `NumpadEnter` key.
    NumpadEnter,
    /// The `NumpadEquals` / `=` key.
    NumpadEquals,
    /// The `Oem102` key.
    Oem102,
    /// The `Period` / `.` key.
    Period,
    /// The `PlayPause` key.
    PlayPause,
    /// The `Power` key.
    Power,
    /// The `PrevTrack` key.
    PrevTrack,

    /// The `Right Alt` key. Maps to `Right Option` on Mac.
    AltRight,
    /// The `Right Bracket` / `]` key.
    BracketRight,
    /// The `Right Control` key.
    ControlRight,
    /// The `Right Shift` key.
    ShiftRight,
    /// The `Right Super` key.
    /// Generic keyboards usually display this key with the *Microsoft Windows* logo.
    /// Apple keyboards call this key the *Command Key* and display it using the ⌘ character.
    #[doc(alias("RWin", "RMeta", "RLogo"))]
    SuperRight,

    /// The `Semicolon` / `;` key.
    Semicolon,
    /// The `Slash` / `/` key.
    Slash,
    /// The `Sleep` key.
    Sleep,
    /// The `Stop` key.
    Stop,
    /// The `NumpadSubtract` / `-` key.
    NumpadSubtract,
    /// The `Sysrq` key.
    Sysrq,
    /// The `Tab` / `   ` key.
    Tab,
    /// The `Underline` / `_` key.
    Underline,
    /// The `Unlabeled` key.
    Unlabeled,

    /// The `VolumeDown` key.
    VolumeDown,
    /// The `VolumeUp` key.
    VolumeUp,

    /// The `Wake` key.
    Wake,

    /// The `WebBack` key.
    WebBack,
    /// The `WebFavorites` key.
    WebFavorites,
    /// The `WebForward` key.
    WebForward,
    /// The `WebHome` key.
    WebHome,
    /// The `WebRefresh` key.
    WebRefresh,
    /// The `WebSearch` key.
    WebSearch,
    /// The `WebStop` key.
    WebStop,

    /// The `Yen` key.
    Yen,

    /// The `Copy` key.
    Copy,
    /// The `Paste` key.
    Paste,
    /// The `Cut` key.
    Cut,

    /// The bottom action button of the action pad (i.e. PS: Cross, Xbox: A).
    GamepadSouth(usize),
    /// The right action button of the action pad (i.e. PS: Circle, Xbox: B).
    GamepadEast(usize),
    /// The upper action button of the action pad (i.e. PS: Triangle, Xbox: Y).
    GamepadNorth(usize),
    /// The left action button of the action pad (i.e. PS: Square, Xbox: X).
    GamepadWest(usize),

    /// The C button.
    GamepadC(usize),
    /// The Z button.
    GamepadZ(usize),

    /// The first left trigger.
    GamepadLeftTrigger(usize),
    /// The second left trigger.
    GamepadLeftTrigger2(usize),
    /// The first right trigger.
    GamepadRightTrigger(usize),
    /// The second right trigger.
    GamepadRightTrigger2(usize),
    /// The select button.
    GamepadSelect(usize),
    /// The start button.
    GamepadStart(usize),
    /// The mode button.
    GamepadMode(usize),

    /// The left thumb stick button.
    GamepadLeftThumb(usize),
    /// The right thumb stick button.
    GamepadRightThumb(usize),

    /// The up button of the D-Pad.
    GamepadDPadUp(usize),
    /// The down button of the D-Pad.
    GamepadDPadDown(usize),
    /// The left button of the D-Pad.
    GamepadDPadLeft(usize),
    /// The right button of the D-Pad.
    GamepadDPadRight(usize),

    /// Miscellaneous buttons, considered non-standard (i.e. Extra buttons on a flight stick that do not have a gamepad equivalent).
    GamepadOther(u8, usize),

    /// The left mouse button.
    MouseLeft,
    /// The right mouse button.
    MouseRight,
    /// The middle mouse button.
    MouseMiddle,
    /// Another mouse button with the associated number.
    MouseOther(u16),

    /// Represents an input button that is not accessible
    /// Most often occurs when there was an error converting to a UniversalInput
    Unknown(u32),
}

impl From<KeyCode> for UniversalInput {
    fn from(value: KeyCode) -> Self {
        // Key names are named exactly as they're key code.
        // Must first convert to physical key location
        let key_str = value.variant_name();
        let Ok(scan_code) = get_scan_code(key_str) else {
            warn!("Error KeyCode -> UniversalInput");
            return UniversalInput::Unknown(0);
        };
        let Ok(scan_code) = get_key(scan_code) else {
            warn!("Error KeyCode -> UniversalInput");
            return UniversalInput::Unknown(scan_code);
        };

        scan_code
    }
}

impl From<ScanCode> for UniversalInput {
    fn from(value: ScanCode) -> Self {
        let Ok(scan_code) = get_key(value.0) else {
            warn!("Error ScanCode -> UniversalInput");
            return UniversalInput::Unknown(value.0);
        };

        scan_code
    }
}

impl From<GamepadButton> for UniversalInput {
    fn from(value: GamepadButton) -> Self {
        let button_type = value.button_type;
        let id = value.gamepad.id;

        match button_type {
            GamepadButtonType::South => UniversalInput::GamepadSouth(id),
            GamepadButtonType::East => UniversalInput::GamepadEast(id),
            GamepadButtonType::North => UniversalInput::GamepadNorth(id),
            GamepadButtonType::West => UniversalInput::GamepadWest(id),
            GamepadButtonType::C => UniversalInput::GamepadC(id),
            GamepadButtonType::Z => UniversalInput::GamepadZ(id),
            GamepadButtonType::LeftTrigger => UniversalInput::GamepadLeftTrigger(id),
            GamepadButtonType::LeftTrigger2 => UniversalInput::GamepadLeftTrigger2(id),
            GamepadButtonType::RightTrigger => UniversalInput::GamepadRightTrigger(id),
            GamepadButtonType::RightTrigger2 => {
                UniversalInput::GamepadRightTrigger2(id)
            }
            GamepadButtonType::Select => UniversalInput::GamepadSelect(id),
            GamepadButtonType::Start => UniversalInput::GamepadStart(id),
            GamepadButtonType::Mode => UniversalInput::GamepadMode(id),
            GamepadButtonType::LeftThumb => UniversalInput::GamepadLeftThumb(id),
            GamepadButtonType::RightThumb => UniversalInput::GamepadRightThumb(id),
            GamepadButtonType::DPadUp => UniversalInput::GamepadDPadUp(id),
            GamepadButtonType::DPadDown => UniversalInput::GamepadDPadDown(id),
            GamepadButtonType::DPadLeft => UniversalInput::GamepadDPadLeft(id),
            GamepadButtonType::DPadRight => UniversalInput::GamepadDPadRight(id),
            GamepadButtonType::Other(c) => UniversalInput::GamepadOther(c, id),
        }
    }
}

impl From<MouseButton> for UniversalInput {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => UniversalInput::MouseLeft,
            MouseButton::Right => UniversalInput::MouseRight,
            MouseButton::Middle => UniversalInput::MouseMiddle,
            MouseButton::Other(id) => UniversalInput::MouseOther(id),
        }
    }
}

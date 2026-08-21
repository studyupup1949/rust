#![warn(clippy::all)]
#![warn(clippy::cargo)]
#![allow(clippy::needless_return, clippy::multiple_crate_versions)]
#![warn(dead_code)]

mod action;
mod action_input;
mod control_scheme;
mod multi_input;
mod multi_scheme;
mod plugin;
mod universal_input;

pub mod prelude {
    pub use crate::actions::Action;
    pub use crate::actions::ActionInput;
    pub use crate::controls::ControlScheme;
    pub use crate::make_controls;
    pub use crate::plugin::ActionMapPlugin;
    pub use crate::plugin::ActionMapSet;
}

pub mod multiplayer_prelude {
    pub use crate::make_multi_input;
    pub use crate::multi_input::MultiInput;
    pub use crate::multi_scheme::MultiScheme;
    pub use crate::plugin::MultiActionMapPlugin;
    pub use crate::prelude::*;
}

pub mod actions {
    pub use crate::action::Action;
    pub use crate::action_input::*;
    pub use crate::multi_input::*;
}

pub mod controls {
    pub use crate::control_scheme::*;
    pub use crate::multi_scheme::*;
}

pub mod input {
    pub use crate::universal_input::*;
}

use thiserror::Error;
use universal_input::UniversalInput;

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("OS {0} is not supported")]
    UnsupportedOs(String),
    #[error("Key {0} is not supported")]
    KeyNotFound(String),
    #[error("ScanCode {0} is not supported")]
    ScanCodeNotFound(String),
}

/// Rudimentary helper function to get the scan code for a key.
/// This will hopefully be rendered obsolete in Bevy 0.13 with
/// [this](https://github.com/bevyengine/bevy/pull/10702) PR.
/// ```rust
/// use action_maps::get_scan_code;
/// let qwerty_w_scan_code = get_scan_code("W");
/// ```
pub fn get_scan_code(key: &str) -> Result<u32, KeyError> {
    match std::env::consts::OS {
        "macos" => match key {
            "," => Ok(0x2B),
            "." => Ok(0x2F),
            "Esc" => Ok(0x35),
            "1" => Ok(0x12),
            "2" => Ok(0x13),
            "3" => Ok(0x14),
            "4" => Ok(0x15),
            "5" => Ok(0x17),
            "6" => Ok(0x16),
            "7" => Ok(0x1A),
            "8" => Ok(0x1C),
            "9" => Ok(0x19),
            "0" => Ok(0x1D),
            "A" => Ok(0x00),
            "S" => Ok(0x01),
            "D" => Ok(0x02),
            "F" => Ok(0x03),
            "H" => Ok(0x04),
            "G" => Ok(0x05),
            "Z" => Ok(0x06),
            "X" => Ok(0x07),
            "C" => Ok(0x08),
            "V" => Ok(0x09),
            "B" => Ok(0x0B),
            "Q" => Ok(0x0C),
            "W" => Ok(0x0D),
            "E" => Ok(0x0E),
            "R" => Ok(0x0F),
            "Y" => Ok(0x10),
            "T" => Ok(0x11),
            "Equal" => Ok(0x18),
            "Minus" => Ok(0x1B),
            "]" => Ok(0x1E),
            "O" => Ok(0x1F),
            "U" => Ok(0x20),
            "[" => Ok(0x21),
            "I" => Ok(0x22),
            "P" => Ok(0x23),
            "Enter" => Ok(0x24),
            "L" => Ok(0x25),
            "J" => Ok(0x26),
            "Quote" => Ok(0x27),
            "K" => Ok(0x28),
            "Semicolon" => Ok(0x29),
            "Backslash" => Ok(0x2A),
            "Comma" => Ok(0x2B),
            "Slash" => Ok(0x2C),
            "N" => Ok(0x2D),
            "M" => Ok(0x2E),
            "Period" => Ok(0x2F),
            "Tab" => Ok(0x30),
            "Space" => Ok(0x31),
            "Backspace" => Ok(0x33),
            "F1" => Ok(0x7A),
            "F2" => Ok(0x78),
            "F4" => Ok(0x76),
            "F5" => Ok(0x60),
            "F6" => Ok(0x61),
            "F7" => Ok(0x62),
            "F3" => Ok(0x63),
            "F8" => Ok(0x64),
            "F9" => Ok(0x65),
            "F11" => Ok(0x67),
            "F12" => Ok(0x6F),
            "Insert" => Ok(0x72),
            "Home" => Ok(0x73),
            "PageUp" => Ok(0x74),
            "Delete" => Ok(0x75),
            "End" => Ok(0x77),
            "PageDown" => Ok(0x79),
            "Left" => Ok(0x7B),
            "Right" => Ok(0x7C),
            "Down" => Ok(0x7D),
            "Up" => Ok(0x7E),
            key => Err(KeyError::KeyNotFound(key.to_string())),
        },
        "windows" => match key {
            "Esc" => Ok(0x01),
            "1" => Ok(0x02),
            "2" => Ok(0x03),
            "3" => Ok(0x04),
            "4" => Ok(0x05),
            "5" => Ok(0x06),
            "6" => Ok(0x07),
            "7" => Ok(0x08),
            "8" => Ok(0x09),
            "9" => Ok(0x0A),
            "0" => Ok(0x0B),
            "-" => Ok(0x0C),
            "=" => Ok(0x0D),
            "Backspace" => Ok(0x0E),
            "Tab" => Ok(0x0F),
            "Q" => Ok(0x10),
            "W" => Ok(0x11),
            "E" => Ok(0x12),
            "R" => Ok(0x13),
            "T" => Ok(0x14),
            "Y" => Ok(0x15),
            "U" => Ok(0x16),
            "I" => Ok(0x17),
            "O" => Ok(0x18),
            "P" => Ok(0x19),
            "[" => Ok(0x1A),
            "]" => Ok(0x1B),
            "Enter" => Ok(0x1C),
            "Ctrl" => Ok(0x1D),
            "A" => Ok(0x1E),
            "S" => Ok(0x1F),
            "D" => Ok(0x20),
            "F" => Ok(0x21),
            "G" => Ok(0x22),
            "H" => Ok(0x23),
            "J" => Ok(0x24),
            "K" => Ok(0x25),
            "L" => Ok(0x26),
            ";" => Ok(0x27),
            "'" => Ok(0x28),
            "`" => Ok(0x29),
            "LShift" => Ok(0x2A),
            "\\" => Ok(0x2B),
            "Z" => Ok(0x2C),
            "X" => Ok(0x2D),
            "C" => Ok(0x2E),
            "V" => Ok(0x2F),
            "B" => Ok(0x30),
            "N" => Ok(0x31),
            "M" => Ok(0x32),
            "," => Ok(0x33),
            "." => Ok(0x34),
            "/" => Ok(0x35),
            "RShift" => Ok(0x36),
            "PtScr" => Ok(0x37),
            "Alt" => Ok(0x38),
            "Space" => Ok(0x39),
            "CpsLk" => Ok(0x3A),
            "F1" => Ok(0x3B),
            "F2" => Ok(0x3C),
            "F3" => Ok(0x3D),
            "F4" => Ok(0x3E),
            "F5" => Ok(0x3F),
            "F6" => Ok(0x40),
            "F7" => Ok(0x41),
            "F8" => Ok(0x42),
            "F9" => Ok(0x43),
            "F10" => Ok(0x44),
            "Num" => Ok(0x45),
            "ScrlLk" => Ok(0x46),
            "Home" => Ok(0x47),
            "Pg" => Ok(0x49),
            "Num-" => Ok(0x4A),
            "Up" => Ok(0xE048),
            "Down" => Ok(0xE050),
            "Left" => Ok(0xE04B),
            "Right" => Ok(0xE04D),
            "NumpadUp" => Ok(0x48),
            "NumpadDown" => Ok(0x50),
            "NumpadLeft" => Ok(0x4B),
            "NumpadRight" => Ok(0x4D),
            "End" => Ok(0xC8),
            "PgDown" => Ok(0x51),
            "Ins" => Ok(0x52),
            "Del" => Ok(0x53),
            key => Err(KeyError::KeyNotFound(key.to_string())),
        },
        os => Err(KeyError::UnsupportedOs(os.to_string())),
    }
}

/// Outputs a UniversalInput given a scancode
pub fn get_key(scan_code: u32) -> Result<UniversalInput, KeyError> {
    match std::env::consts::OS {
        "macos" => match scan_code {
            0x2B => Ok(UniversalInput::Comma),
            0x2F => Ok(UniversalInput::Period),
            0x35 => Ok(UniversalInput::Escape),
            0x12 => Ok(UniversalInput::Key1),
            0x13 => Ok(UniversalInput::Key2),
            0x14 => Ok(UniversalInput::Key3),
            0x15 => Ok(UniversalInput::Key4),
            0x17 => Ok(UniversalInput::Key5),
            0x16 => Ok(UniversalInput::Key6),
            0x1A => Ok(UniversalInput::Key7),
            0x1C => Ok(UniversalInput::Key8),
            0x19 => Ok(UniversalInput::Key9),
            0x1D => Ok(UniversalInput::Key0),
            0x00 => Ok(UniversalInput::A),
            0x01 => Ok(UniversalInput::S),
            0x02 => Ok(UniversalInput::D),
            0x03 => Ok(UniversalInput::F),
            0x04 => Ok(UniversalInput::H),
            0x05 => Ok(UniversalInput::G),
            0x06 => Ok(UniversalInput::Z),
            0x07 => Ok(UniversalInput::X),
            0x08 => Ok(UniversalInput::C),
            0x09 => Ok(UniversalInput::V),
            0x0B => Ok(UniversalInput::B),
            0x0C => Ok(UniversalInput::Q),
            0x0D => Ok(UniversalInput::W),
            0x0E => Ok(UniversalInput::E),
            0x0F => Ok(UniversalInput::R),
            0x10 => Ok(UniversalInput::Y),
            0x11 => Ok(UniversalInput::T),
            0x18 => Ok(UniversalInput::Equals),
            0x1B => Ok(UniversalInput::Minus),
            0x1F => Ok(UniversalInput::O),
            0x20 => Ok(UniversalInput::U),
            0x22 => Ok(UniversalInput::I),
            0x23 => Ok(UniversalInput::P),
            0x24 => Ok(UniversalInput::Return),
            0x25 => Ok(UniversalInput::L),
            0x26 => Ok(UniversalInput::J),
            0x27 => Ok(UniversalInput::Apostrophe),
            0x28 => Ok(UniversalInput::K),
            0x29 => Ok(UniversalInput::Semicolon),
            0x2A => Ok(UniversalInput::Backslash),
            0x2C => Ok(UniversalInput::Slash),
            0x2D => Ok(UniversalInput::N),
            0x2E => Ok(UniversalInput::M),
            0x30 => Ok(UniversalInput::Tab),
            0x31 => Ok(UniversalInput::Space),
            0x33 => Ok(UniversalInput::Back),
            0x7A => Ok(UniversalInput::F1),
            0x78 => Ok(UniversalInput::F2),
            0x76 => Ok(UniversalInput::F4),
            0x60 => Ok(UniversalInput::F5),
            0x61 => Ok(UniversalInput::F6),
            0x62 => Ok(UniversalInput::F7),
            0x63 => Ok(UniversalInput::F3),
            0x64 => Ok(UniversalInput::F8),
            0x65 => Ok(UniversalInput::F9),
            0x67 => Ok(UniversalInput::F11),
            0x6F => Ok(UniversalInput::F12),
            0x72 => Ok(UniversalInput::Insert),
            0x73 => Ok(UniversalInput::Home),
            0x74 => Ok(UniversalInput::PageUp),
            0x75 => Ok(UniversalInput::Delete),
            0x77 => Ok(UniversalInput::End),
            0x79 => Ok(UniversalInput::PageDown),
            0x7B => Ok(UniversalInput::Left),
            0x7C => Ok(UniversalInput::Right),
            0x7D => Ok(UniversalInput::Down),
            0x7E => Ok(UniversalInput::Up),
            sc => Err(KeyError::ScanCodeNotFound(sc.to_string())),
        },
        "windows" => match scan_code {
            0x01 => Ok(UniversalInput::Escape),
            0x02 => Ok(UniversalInput::Key1),
            0x03 => Ok(UniversalInput::Key2),
            0x04 => Ok(UniversalInput::Key3),
            0x05 => Ok(UniversalInput::Key4),
            0x06 => Ok(UniversalInput::Key5),
            0x07 => Ok(UniversalInput::Key6),
            0x08 => Ok(UniversalInput::Key7),
            0x09 => Ok(UniversalInput::Key8),
            0x0A => Ok(UniversalInput::Key9),
            0x0B => Ok(UniversalInput::Key0),
            0x0C => Ok(UniversalInput::Minus),
            0x0D => Ok(UniversalInput::Equals),
            0x0E => Ok(UniversalInput::Back),
            0x0F => Ok(UniversalInput::Tab),
            0x10 => Ok(UniversalInput::Q),
            0x11 => Ok(UniversalInput::W),
            0x12 => Ok(UniversalInput::E),
            0x13 => Ok(UniversalInput::R),
            0x14 => Ok(UniversalInput::T),
            0x15 => Ok(UniversalInput::Y),
            0x16 => Ok(UniversalInput::U),
            0x17 => Ok(UniversalInput::I),
            0x18 => Ok(UniversalInput::O),
            0x19 => Ok(UniversalInput::P),
            0x1E => Ok(UniversalInput::A),
            0x1F => Ok(UniversalInput::S),
            0x20 => Ok(UniversalInput::D),
            0x21 => Ok(UniversalInput::F),
            0x22 => Ok(UniversalInput::G),
            0x23 => Ok(UniversalInput::H),
            0x24 => Ok(UniversalInput::J),
            0x25 => Ok(UniversalInput::K),
            0x26 => Ok(UniversalInput::L),
            0x27 => Ok(UniversalInput::Semicolon),
            0x28 => Ok(UniversalInput::Apostrophe),
            0x2B => Ok(UniversalInput::Backslash),
            0x2C => Ok(UniversalInput::Z),
            0x2D => Ok(UniversalInput::X),
            0x2E => Ok(UniversalInput::C),
            0x2F => Ok(UniversalInput::V),
            0x30 => Ok(UniversalInput::B),
            0x31 => Ok(UniversalInput::N),
            0x32 => Ok(UniversalInput::M),
            0x33 => Ok(UniversalInput::Comma),
            0x34 => Ok(UniversalInput::Period),
            0x35 => Ok(UniversalInput::Slash),
            0x39 => Ok(UniversalInput::Space),
            0x3B => Ok(UniversalInput::F1),
            0x3C => Ok(UniversalInput::F2),
            0x3D => Ok(UniversalInput::F3),
            0x3E => Ok(UniversalInput::F4),
            0x3F => Ok(UniversalInput::F5),
            0x40 => Ok(UniversalInput::F6),
            0x41 => Ok(UniversalInput::F7),
            0x42 => Ok(UniversalInput::F8),
            0x43 => Ok(UniversalInput::F9),
            0x44 => Ok(UniversalInput::F10),
            0x47 => Ok(UniversalInput::Home),
            0xC8 => Ok(UniversalInput::End),
            sc => Err(KeyError::ScanCodeNotFound(sc.to_string())),
        },
        os => Err(KeyError::UnsupportedOs(os.to_string())),
    }
}

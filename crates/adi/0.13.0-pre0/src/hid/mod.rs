// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

//! This HID interface is designed for easy portability.
//!
//!

mod ffi;

pub(crate) use self::ffi::NativeManager;

/// PC Specific Data
///
/// This is necessary for:
/// * emulating LStick and RStick on a PC.
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "bitrig",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "windows",
    target_os = "macos",
    target_os = "redox",
    target_os = "deskron",
    target_arch = "wasm32",
))]
pub(crate) mod pc {
    /// Storage for whether or not keys are pressed in emulated joysticks.
    static mut JOYKEYS: [u8; 2] = [0, 0];

    /// W/I Key "Up" on LStick/RStick: (0.0, -1.0)
    pub(crate) const WI: u8 = 0b_0000_1000;
    /// A/J Key "Left" on LStick/RStick: (-1.0, 0.0)
    pub(crate) const AJ: u8 = 0b_0000_0100;
    /// S/K Key "Down" on LStick/RStick: (0.0, 1.0)
    pub(crate) const SK: u8 = 0b_0000_0010;
    /// D/L Key "Right" on LStick/RStick: (1.0, 0.0)
    pub(crate) const DL: u8 = 0b_0000_0001;

    /// Set W,A,S or D on the WASD Mask (Key Press).
    #[inline(always)]
    pub(crate) fn wasd_set(mask: u8) -> () {
        match mask {
            WI => super::set_lstick(0, 0.0, -1.0),
            AJ => super::set_lstick(0, -1.0, 0.0),
            SK => super::set_lstick(0, 0.0, 1.0),
            DL => super::set_lstick(0, 1.0, 0.0),
            _ => panic!("Mask is bad."),
        }
        unsafe { JOYKEYS[0] |= mask }
    }

    /// Unset W,A,S or D on the WASD Mask (Key Release).  LStick will be reset to (0.0, 0.0) only
    /// if none of W,A,S or D are held down.
    #[inline(always)]
    pub(crate) fn wasd_unset(mask: u8) -> () {
        unsafe { JOYKEYS[0] &= !mask }
        if unsafe { JOYKEYS[0] == 0 } {
            super::set_lstick(0, 0.0, 0.0)
        }
    }

    /// Set W,A,S or D on the WASD Mask (Key Press).
    #[inline(always)]
    pub(crate) fn ijkl_set(mask: u8) -> () {
        match mask {
            WI => super::set_rstick(0, 0.0, -1.0),
            AJ => super::set_rstick(0, -1.0, 0.0),
            SK => super::set_rstick(0, 0.0, 1.0),
            DL => super::set_rstick(0, 1.0, 0.0),
            _ => panic!("Mask is bad."),
        }
        unsafe { JOYKEYS[1] |= mask }
    }

    /// Unset W,A,S or D on the WASD Mask (Key Release).  LStick will be reset to (0.0, 0.0) only
    /// if none of W,A,S or D are held down.
    #[inline(always)]
    pub(crate) fn ijkl_unset(mask: u8) -> () {
        unsafe { JOYKEYS[1] &= !mask }
        if unsafe { JOYKEYS[1] == 0 } {
            super::set_rstick(0, 0.0, 0.0)
        }
    }
}

/*/// A type for mapping platform-dependant input to platform-independant input.
///
/// ```
/// # Computer
///

/// Directionals (Controls with 8 options corresponding to directions)
/// 0. WASD
/// 2. Numpad
/// 3. Arrow Keys (Interpolation)
/// 4. Mouse Motion (Mouse as Stick)
/// Common Controls
/// 0. Enter
/// 1. Space
/// 2. Backspace
/// 3. Tab
/// 4. CapsLock
/// 5. Escape
/// 6. LShift
/// 7. RShift
/// 8. BackSlash
/// Consistent Controls
/// 0. Ctrl => Cmd
/// 1. LAlt => Menu
/// 2. Esc => Back
///
/// # Phone / Tablet
/// 0. Region (Split into 9 Regions)
/// 1. Virtual LStick
/// 2. Virtual RStick
/// 3. Touch Motion (Drag)
/// 4. Virtual NumPad
///
///
/// # Game System
/// 0. LStick
/// 1. RStick
/// 2. ABXY & Compbinations
/// 3. DPad
/// 4. Touch Motion (Drag)
///
/// # Drop
/// 0. LStick
/// 1. RStick
/// 2. NumPad
/// 3. Arrow Pad
/// 4. Touch Motion (Drag)
///
/// Move => LStick or WASD, depending on platform
/// Camera => RStick or MouseAsStick(for 3D games, etc.)
/// Actions => ABXY combinations or Numpad
/// Arrows =>
/// ```
pub enum Mapper {
    /// Joystick 1
    StickA,
}*/

// 128 bits memory for simulating one unified HID.
static mut HID_STATE: Option<Vec<HidState>> = None;

/// Input button for `HidState`.
///
/// Controllers with more buttons act as combinational buttons:
/// * ZL = Cmd/Z + L
/// * ZR = Cmd/Z + R
/// * Minus = Cmd/Z + Plus (Nintendo Switch)
#[repr(u64)]
#[derive(Clone)]
pub enum Key {
    /// Up Arrow Key / DPad Up
    Up = 0b__0000_0000__0000_0001,
    /// Left Arrow Key / DPad Left
    Left = 0b__0000_0000__0000_0010,
    /// Right Arrow Key / DPad Right
    Right = 0b__0000_0000__0000_0100,
    /// Down Arrow Key / DPad Down
    Down = 0b__0000_0000__0000_1000,

    /// Space Key / X Button
    Execute = 0b__0000_0000__0001_0000,
    /// Tab Key / Y Button
    Action = 0b__0000_0000__0010_0000,
    /// Enter Key / A Button
    Accept = 0b__0000_0000__0100_0000,
    /// Shift Key / B Button
    Control = 0b__0000_0000__1000_0000,

    /// Escape / Start-Pause-Menu Button
    Back = 0b__0000_0001__0000_0000,
    /// Ctrl Key / Z Button
    Cmd = 0b__0000_0010__0000_0000,
    /// Scroll Up / L Button
    L = 0b__0000_0100__0000_0000,
    /// Scroll Down / R Button
    R = 0b__0000_1000__0000_0000,

    /// Touch or Click
    Press = 0b__0001_0000__0000_0000,
}

impl Key {
    /// A very simple "get whether or not a key is currently being held down".
    #[inline(always)]
    pub fn held(&self, controller: usize) -> bool {
        let input = unsafe { HID_STATE.as_mut().unwrap()[controller].input };

        (input & (self.clone() as u64)) != 0
    }

    // Get whether or not a key has just been modified.
    #[inline(always)]
    fn just(&self, controller: usize) -> (bool, bool) {
        let memory = unsafe { HID_STATE.as_mut().unwrap()[controller].memory };

        let mem = (memory & (self.clone() as u64)) != 0;
        let get = self.held(controller);

        (get, mem ^ get)
    }

    /// Get whether a press "event" has just happenned.
    pub fn pressed(&self, controller: usize) -> bool {
        self.just(controller) == (true, true)
    }

    /// Get whether a release "event" has just happenned.
    pub fn released(&self, controller: usize) -> bool {
        self.just(controller) == (false, true)
    }
}

/// Different Outputs for HID.
#[repr(u32)]
enum Output {
    /// Vibrate Controller in the HID.
    HapticStart = 0b__0000_0000__0000_0000__0000_0000__0000_0001,
    /// Vibrate Controller in the HID.
    HapticStop = 0b__0000_0000__0000_0000__0000_0000__0000_0010,
}

/// An abstract input state which encompasses all HID's.
///
/// # Abstraction over platform differences, explained.
/// Here is the default settings for which keys are equivalent to which across platform boundaries.
/// > * Arrow Keys = D Pad
/// > * Enter,Shift,Space,Tab = ABXY
/// > * Escape Key = Back Key
/// > * Cmd = Ctrl
/// > * WASD = Move Stick
/// > * IJKL = Camera Stick
/// > * Scroll Up = L
/// > * Scroll Down = R
// 8 * 64 bits (512 bits = 64 bytes).
#[repr(C)]
#[derive(Clone)]
pub(crate) struct HidState {
    /// 32 bits memory for simulating unicode text input.
    text: char,
    /// Output.
    output: u32,

    /// X Range: [? .. ?], based on aspect ratio.
    pub screen_x: f32,
    /// Y Range: [-1 .. 1].
    pub screen_y: f32,

    /// Input: Binary key states.
    pub input: u64,

    /// Input: Memory
    pub memory: u64,

    /// Left C-Pad X, guaranteed Range [-1 .. 1]
    pub lstick_x: f32,
    /// Left C-Pad Y, guaranteed Range [-1 .. 1]
    pub lstick_y: f32,

    /// Right C-Pad X, guaranteed Range [-1 .. 1]
    pub rstick_x: f32,
    /// Right C-Pad Y, guaranteed Range [-1 .. 1]
    pub rstick_y: f32,

    /// Left throttle
    pub l_throttle: f32,
    /// Right throttle
    pub r_throttle: f32,

    /// Extension Left / Left-Right Axis
    pub ext_axis_a: f32,
    /// Extension Right / Up-Down Axis
    pub ext_axis_b: f32,
}

impl HidState {
    // Set a key true.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn key_press(&mut self, key: Key) {
        self.input |= key as u64;
    }

    // Set a key false.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn key_release(&mut self, key: Key) {
        self.input &= !(key as u64);
    }

    // Set a key true.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn ext_press(&mut self, key: u8) {
        assert!(key < 48);
        self.input |= (1u64 << 63) >> key;
    }

    // Set a key false.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn ext_release(&mut self, key: u8) {
        assert!(key < 48);
        self.input &= !((1u64 << 63) >> key);
    }
}

// ////////////////////////////////////////////////////////////////////////////////////////// //
//                                   Public Functions                                         //
// ////////////////////////////////////////////////////////////////////////////////////////// //

/// Get the Left Stick X & Y (WASD).  Ranges are [-1 .. 1].
pub fn lstick(controller: usize) -> (f32, f32) {
    unsafe {
        (
            HID_STATE.as_mut().unwrap()[controller].lstick_x,
            HID_STATE.as_mut().unwrap()[controller].lstick_y,
        )
    }
}

/// Get the Right Stick X & Y (Mouse Movement).  Ranges are [-1 .. 1].
pub fn rstick(controller: usize) -> (f32, f32) {
    unsafe {
        (
            HID_STATE.as_mut().unwrap()[controller].rstick_x,
            HID_STATE.as_mut().unwrap()[controller].rstick_y,
        )
    }
}

/// Get text input (One character per frame max, \0 if nothing).
pub fn text(controller: usize) -> char {
    unsafe { HID_STATE.as_mut().unwrap()[controller].text }
}

/// Get screen X.
pub fn screen_x(controller: usize) -> f32 {
    unsafe { HID_STATE.as_mut().unwrap()[controller].screen_x }
}

/// Get screen Y.
pub fn screen_y(controller: usize) -> f32 {
    unsafe { HID_STATE.as_mut().unwrap()[controller].screen_y }
}

// ////////////////////////////////////////////////////////////////////////////////////////////// //
//                                   Private Functions                                            //
// ////////////////////////////////////////////////////////////////////////////////////////////// //

pub(crate) fn new() -> NativeManager {
    unsafe {
        HID_STATE = Some(vec![HidState {
            text: '\0',
            output: 0,
            screen_x: 0.0,
            screen_y: 0.0,
            input: 0,
            memory: 0,

            lstick_x: 0.0,
            lstick_y: 0.0,
            rstick_x: 0.0,
            rstick_y: 0.0,
            l_throttle: 0.0,
            r_throttle: 0.0,
            ext_axis_a: 0.0,
            ext_axis_b: 0.0,
        }]);
    }

    NativeManager::new()
}

// Reset, and get new input.
#[inline(always)]
pub(crate) fn update(c_manager: &mut NativeManager) {
    let (device_count, added) = c_manager.search();

    if added != ::std::usize::MAX {
        unsafe {
            HID_STATE.as_mut().unwrap().resize(
                device_count,
                HidState {
                    text: '\0',
                    output: 0,
                    screen_x: 0.0,
                    screen_y: 0.0,
                    input: 0,
                    memory: 0,

                    lstick_x: 0.0,
                    lstick_y: 0.0,
                    rstick_x: 0.0,
                    rstick_y: 0.0,
                    l_throttle: 0.0,
                    r_throttle: 0.0,
                    ext_axis_a: 0.0,
                    ext_axis_b: 0.0,
                },
            );
        }
    }

    for i in 0..device_count {
        // Copy old input into memory
        unsafe {
            HID_STATE.as_mut().unwrap()[i].memory = HID_STATE.as_mut().unwrap()[i].input;
        }

        // Check if it's unplugged
        let (fd, is_out, ne) = c_manager.get_fd(i);

        if ne {
            continue;
        }
        if is_out {
            c_manager.disconnect(fd);
            continue;
        }

        unsafe {
            c_manager.poll_event(i, &mut HID_STATE.as_mut().unwrap()[i]);
        }
    }
}

// Set a key true.
#[allow(unused)]
#[inline(always)]
pub(crate) fn key_press(controller: usize, key: Key) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].key_press(key);
    }
}

// Set a key false.
#[allow(unused)]
#[inline(always)]
pub(crate) fn key_release(controller: usize, key: Key) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].key_release(key);
    }
}

// Set a key true.
#[allow(unused)]
#[inline(always)]
pub(crate) fn ext_press(controller: usize, key: u8) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].ext_press(key);
    }
}

// Set a key false.
#[allow(unused)]
#[inline(always)]
pub(crate) fn ext_release(controller: usize, key: u8) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].ext_release(key);
    }
}

// Set left stick.  `x` & `y` are clamped [-1 .. 1].
#[allow(unused)]
#[inline(always)]
pub(crate) fn set_lstick(controller: usize, x: f32, y: f32) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].lstick_x = x.min(1.0).max(-1.0);
        HID_STATE.as_mut().unwrap()[controller].lstick_y = y.min(1.0).max(-1.0);
    }
}

// Set right stick.  `x` & `y` are clamped [-1 .. 1].
#[allow(unused)]
#[inline(always)]
pub(crate) fn set_rstick(controller: usize, x: f32, y: f32) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].rstick_x = x.min(1.0).max(-1.0);
        HID_STATE.as_mut().unwrap()[controller].rstick_y = y.min(1.0).max(-1.0);
    }
}

// Set left throttle.
#[allow(unused)]
#[inline(always)]
pub(crate) fn set_lthrottle(controller: usize, x: f32) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].l_throttle = x.min(1.0).max(-1.0);
    }
}

// Set right throttle.
#[allow(unused)]
#[inline(always)]
pub(crate) fn set_rthrottle(controller: usize, x: f32) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].r_throttle = x.min(1.0).max(-1.0);
    }
}

// Set unicode character input.
#[allow(unused)]
#[inline(always)]
pub(crate) fn set_char(controller: usize, x: char) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].text = x;
    }
}

/*// Get the haptic feedback bit.
#[allow(unused)]
#[inline(always)]
pub(crate) fn get_haptic(controller: usize) -> bool {
    let output = unsafe { HID_STATE.as_mut().unwrap()[controller].output };

    (output & (Output::HapticStart as u32)) != 0
}*/

/// Start a haptic rumble effect (vibrate).
pub fn rumble_start(controller: usize) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].output |= Output::HapticStart as u32;
    }
}

/// Stop a haptic rumble effect (vibrate).
pub fn rumble_stop(controller: usize) {
    unsafe {
        HID_STATE.as_mut().unwrap()[controller].output |= Output::HapticStop as u32;
    }
}

pub(crate) trait CoordToFloat {
    fn to_f32(self) -> f32;
}

impl CoordToFloat for u16 {
    fn to_f32(self) -> f32 {
        self as f32
    }
}

impl CoordToFloat for i16 {
    fn to_f32(self) -> f32 {
        self as f32
    }
}

#[cfg(feature = "screen")]
pub(crate) fn cursor_coordinates<T, U>(wh: (T, T), xy: (U, U))
where
    U: CoordToFloat,
    T: CoordToFloat,
{
    let x = xy.0.to_f32();
    let y = xy.1.to_f32();
    let w = wh.0.to_f32();
    let h = wh.1.to_f32();
    let xy = (x * 2.0 / w - 1.0, y * 2.0 / h - 1.0);

    if xy.0 > 1.0 || xy.0 < -1.0 || xy.1 > 1.0 || xy.1 < -1.0 {
    } else {
        unsafe {
            HID_STATE.as_mut().unwrap()[0].screen_x = xy.0;
            HID_STATE.as_mut().unwrap()[0].screen_y = xy.1;
        }
    }
}

// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

/// Personal Computer Input for Direction.
pub enum DirInputPC {
    /// Personal Computer: Get directional region of computer screen left click.
    /// 
    /// Touch Screen: Get directional region of phone touched.
    Region,
    /// Up, Down, Left, Right Arrow Keys, plus combinations.
    ///
    /// Game Controller: DPad.
    ///
    /// Phone: Virtual DPad (Graphical).
    ArrowKeys,
    /// Personal Computer: WASD / QWEASDZC(X).
    /// 
    /// Game Controller: LStick
    Left,
    /// Number Pad Keys.
    NumPad,
    /// Mouse moves (For looking around in 3D, generally used in games).
    Motion,
}

/// Phone Input for Direction.
pub enum DirInputTouch {
    /// 
    Region,



    /// A Virtual LStick (Graphical).
    Left,
    /// A Virtual RStick (Graphical).
    Right,
    /// A Virtual Number Pad (Graphical).
    NumPad,
    /// Touch and Drag.
    Motion,
}

/// Controller Input for Direction.
pub enum DirInputController {
    /// Up, Down, Left, Right, plus combinations.  May be a DPad.
    ArrowKeys,
    /// A Virtual LStick (Graphical).
    Left,
    /// A Virtual RStick (Graphical).
    Right,
    /// A Virtual Number Pad (Graphical), Also, AXBY Combinations.
    NumPad,
    /// Touch and Drag.
    Motion,
}

/// A DirInput object.
pub struct DirInput {
    /// Computer Specific Input
    pub computer: DirInputComputer,
    /// Touchscreen Specific Input
    pub phone: DirInputPhone,
    /// Controller Specific Input.
    pub controller: DirInputController,
}

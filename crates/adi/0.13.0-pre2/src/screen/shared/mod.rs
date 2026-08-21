// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use ami::Matrix;

/// GUI Texture Coordinates.
pub(crate) const GUI_TC: [f32; 16] = [
    0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
];

/// GUI Model Coordinates.
pub(crate) const GUI_MC: [f32; 16] = [
    -1.0, -1.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0,
];

/// GUI Model Coordinates Fans.
pub(crate) fn gui_mc_fans() -> Vec<(u32, u32)> {
    vec![(0, 4)]
}

/// Generate a projection matrix.
pub(crate) fn projection(ratiox: f32, fovy: f32) -> Matrix {
    matrix!()
        .m(Matrix::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ))
        .m(Matrix::finite_perspective_projection(
            fovy, ratiox, 0.1,   // Near
            100.0, // Far
        ))
}

// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use std::os::raw::c_void;

use ami::Matrix;

use super::PlatformDependant;

type BargViewer = Box<[f32; 16]>;

// Put the linked Matrix into storage (location is implementation dependant)
pub(crate) fn viewer_new(mat: [f32; 16]) -> *mut c_void {
    let viewer: BargViewer = BargViewer::new(mat);

    BargViewer::into_raw(viewer) as *mut _
}

// Set the linked Matrix.
pub(crate) fn viewer_set(viewer: &mut *mut c_void, mat: [f32; 16]) {
    let mut viewer2: BargViewer = unsafe { BargViewer::from_raw((*viewer) as *mut _) };

    *viewer2 = mat;

    *viewer = BargViewer::into_raw(viewer2) as *mut _;
}

// Clean up the linked Matrix.
pub(crate) fn viewer_old(viewer: *mut c_void) {
    let _viewer: BargViewer = unsafe { BargViewer::from_raw(viewer as *mut _) };
}

/*// Put the Texture into storage  (location is implementation dependant)
pub(crate) fn texture_new(wh: (u16, u16), tex: &[u8]) -> *mut c_void {
    let texture: BargTexture = BargTexture::new(wh, tex);

    BargTexture::into_raw(texture) as *mut _
}

// Set the Texture.
pub(crate) fn texture_set(texture: &mut *mut c_void, writer: &mut FnMut(*mut u8) -> ()) {
}

// Free Texture from storage.
pub(crate) fn texture_old(texture: *mut c_void) {
}*/

pub(crate) fn projection() -> Matrix {
    matrix!()
}

// Function loader for barg.
pub(crate) fn load_functions() -> PlatformDependant {
    PlatformDependant {
        // 0. Barg
        projection,
        // 1. Camera
        viewer_new,
        viewer_set,
        viewer_old,
    }
}

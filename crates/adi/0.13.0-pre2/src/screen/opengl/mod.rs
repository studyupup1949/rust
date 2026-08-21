// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use std::os::raw::c_void;

use ami::Matrix;

use super::ffi::render::opengl::OPENGL;
use super::PlatformDependant;

type OpenGLViewer = Box<usize>;

// Put the linked Matrix into storage (location is implementation dependant)
fn viewer_new(mat: [f32; 16]) -> *mut c_void {
    let opengl = unsafe { OPENGL.clone().unwrap() };

    let viewer: OpenGLViewer = OpenGLViewer::new(opengl.get().fakebuffer.len());

    opengl.get_mut().fakebuffer.push(mat);

    OpenGLViewer::into_raw(viewer) as *mut _
}

// Set the linked Matrix.
fn viewer_set(viewer: &mut *mut c_void, mat: [f32; 16]) {
    let opengl = unsafe { OPENGL.clone().unwrap() };

    let viewer2: OpenGLViewer = unsafe { OpenGLViewer::from_raw((*viewer) as *mut _) };

    opengl.get_mut().fakebuffer[*viewer2] = mat;

    *viewer = OpenGLViewer::into_raw(viewer2) as *mut _;
}

// Clean up the linked Matrix.
fn viewer_old(viewer: *mut c_void) {
    unsafe { OpenGLViewer::from_raw(viewer as *mut _) };
}

fn projection() -> Matrix {
    let opengl = unsafe { OPENGL.clone().unwrap() };

    opengl.projection_get()
}

// Function loader for barg.
pub(crate) fn load_functions() -> PlatformDependant {
    PlatformDependant {
        // 0. OpenGL
        projection,
        // 1. Camera
        viewer_new,
        viewer_set,
        viewer_old,
    }
}

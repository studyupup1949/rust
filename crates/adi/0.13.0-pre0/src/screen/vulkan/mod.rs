// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use std::os::raw::c_void;

use ami::Matrix;

use super::ffi::render::vulkan::{Buffer, BufferBuilderType, VULKAN};
use super::PlatformDependant;

type VulkanViewer = Box<Buffer>;

// Put the linked Matrix into storage (location is implementation dependant)
fn viewer_new(mat: [f32; 16]) -> *mut c_void {
    let viewer: VulkanViewer = VulkanViewer::new(Buffer::new(&mat, BufferBuilderType::Uniform));

    VulkanViewer::into_raw(viewer) as *mut _
}

// Set the linked Matrix.
fn viewer_set(viewer: &mut *mut c_void, mat: [f32; 16]) {
    let viewer2: VulkanViewer = unsafe { VulkanViewer::from_raw((*viewer) as *mut _) };

    viewer2.update(&mat);

    ::std::mem::forget(viewer2);
}

// Clean up the linked Matrix.
fn viewer_old(viewer: *mut c_void) {
    let _viewer: VulkanViewer = unsafe { VulkanViewer::from_raw(viewer as *mut _) };
}

fn projection() -> Matrix {
    let vulkan = unsafe { VULKAN.clone().unwrap() };

    vulkan.projection_get()
}

// Function loader for barg.
pub(crate) fn load_functions() -> PlatformDependant {
    PlatformDependant {
        // 0. Vulkan
        projection,
        // 1. Viewer
        viewer_new,
        viewer_set,
        viewer_old,
    }
}

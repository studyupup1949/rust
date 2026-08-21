# ADI / Screen Feature
This README documents thoughts behind development.  This file may be useful for adding support for
other platforms / rendering engines.

## Types Of Data Structures Needed For Rendering
This list should be kept as simple as possible.

### 1. Camera
A camera is a matrix that can be linked to multiple `Shapes`.

#### Functions:
```
/// Put the linked Matrix into storage (location is implementation dependant)
fn camera_new(mat: [f32; 16]) -> *mut c_void;

/// Set the linked Matrix.
fn camera_set(camera: *mut c_void, mat: [f32; 16]);

/// Clean up the linked Matrix.
fn camera_old(camera: *mut c_void);
```

# [Aldaron's Device Interface / Screen](https://crates.io/crates/adi_screen)
Render graphics to a computer or phone screen, and get input.  Great for both
video games and apps!

## Features
* Create a window
* Render graphics with sprites
* Obtain user input
* Sprites auto depth-sort for fast rendering.
* Text Rendering
* Switch between OpenGL, OpenGLES or Vulkan depending on what's available.
* Switch between XCB, or WinAPI depending on what's available.

## [Contributing](http://plopgrizzly.com/contributing/en#contributing)

## Roadmap to 1.0 (Future Features)
* Custom shaders
* Fix Windows touchpad not working for scroll events.
* Support MacOS + Metal/(or MoltenVK?)
* Support Android + OpenGLES
* Support Android + Vulkan
* Support Touchscreen on Windows 
* Support Touchscreen on Linux w/ XCB
* Support Wayland + OpenGLES & Vulkan
* Support Touchscreen on Linux w/ Wayland
* Support Raspberry Pi Direct To Display + Vulkan
* Support Web Assembly + WebGL

## Change Log
### 0.10
* Octree support is no longer built in to this library.  If you need it, use the
Cala Physics Engine instead or directly depend on AMI.
* Update to newest adi_gpu.

### 0.9
* Fixed `sprites_fog!()` and `sprites_gui!()` macros; they're now like `sprite`.

### 0.8
* Easy model generation.

## Developed by [Plop Grizzly](http://plopgrizzly.com)

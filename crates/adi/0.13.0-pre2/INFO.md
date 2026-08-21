# [Aldaron's Device Interface](https://crates.io/crates/adi)

## Multi-Platform Graphics
* Linux Desktop: `XCB` (`OpenGL or OpenGL(ES) or Vulkan`)
 - XCB works because Wayland is not widespread yet, and where it's used XWayland takes care of it
* Windows: `WinAPI` (`OpenGL or Vulkan`)
 - Windows has only the one option.
* Raspberry Pi: `DirectFB` (`Vulkan`)
 - Raspberry Pi uses DirectFB because then an X server doesn't have to be running, saving CPU time.
* MacOS / iOS: `Cocoa` (`Vulkan (MoltenVK)`)
 - This is the newest Window Manager library for Macs/iOS.
* Android (`OpenGL(ES) or Vulkan`)
* Nintendo Switch (`Vulkan`)
* Web Assembly (`OpenGL`)
* Maybe more in the future.

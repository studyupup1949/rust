#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::Semaphore;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::Semaphore;

#[cfg(not(any(target_os = "linux", target_os = "windows",)))]
compile_error!("abstract_semaphore does not support this operating system.");

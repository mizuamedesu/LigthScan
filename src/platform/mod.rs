#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub mod elevation;

#[cfg(target_os = "windows")]
pub mod module;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows as current;

#[cfg(any(target_os = "macos", test))]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos as current;

#[cfg(any(target_os = "linux", test))]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux as current;

#[cfg(test)]
mod parity_tests;

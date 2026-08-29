//! OS-level numpad digit probe for custom shortcut capture.
//!
//! GPUI names numpad digits like their main-row counterparts (`6` not `kp_6`).
//! A passive platform listener records recent physical keypad presses so the
//! shortcut recorder can map them to HID `Kp*` usages.

mod state;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod stub;
#[cfg(target_os = "windows")]
mod windows;

pub use state::disambiguate;

/// Starts the passive OS listener. Idempotent.
pub fn ensure_running() {
    #[cfg(target_os = "linux")]
    linux::start();
    #[cfg(target_os = "macos")]
    macos::start();
    #[cfg(target_os = "windows")]
    windows::start();
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    stub::start();
}

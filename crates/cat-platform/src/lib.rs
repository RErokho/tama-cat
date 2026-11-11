//! Windows platform: layered pet window, displays, window-top enumeration.

mod displays;
mod layered;
mod perception;
mod windows_enum;

pub use displays::list_displays;
pub use layered::{PetWindow, WindowEvent};
pub use perception::PerceptionSensor;
pub use windows_enum::list_window_surfaces;

pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

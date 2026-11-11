//! Platform-agnostic user activity snapshot (no WinAPI types).

#[derive(Debug, Clone, Copy, Default)]
pub struct UserActivitySnapshot {
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_dx: f32,
    pub cursor_dy: f32,
    pub cursor_speed: f32,
    /// Approximate key events per second over the last sample window.
    pub typing_rate: f32,
    pub typing: bool,
    pub enter_pressed: bool,
    pub backspace_burst: bool,
    pub caps_lock_toggled: bool,
    pub ctrl_s: bool,
    pub ctrl_z: bool,
    pub esc_pressed: bool,
    /// Vertical scroll proxy (raw wheel ticks or vertical cursor sweep).
    pub scroll_delta: f32,
    pub idle_secs: f32,
    /// True on the frame the user returns from idle.
    pub returned_from_idle: bool,
    pub foreground_changed: bool,
    pub window_opened: bool,
    pub window_closed: bool,
    pub session_locked: bool,
    pub session_unlocked: bool,
    pub virtual_desktop_changed: bool,
    pub volume_changed: bool,
    pub clipboard_changed: bool,
    /// Heuristic: drag-select / long click with cursor move while not typing fast.
    pub selection_likely: bool,
    pub selection_large: bool,
    pub cursor_offscreen: bool,
    pub cursor_near_pet: bool,
    pub cursor_in_corner: bool,
    pub print_screen: bool,
    pub right_mouse: bool,
    pub left_double_click_pet: bool,
    pub file_drag_likely: bool,
    pub hour_of_day: u8,
    pub dark_theme_likely: bool,
    pub toast_likely: bool,
    pub error_ui_likely: bool,
    pub loading_likely: bool,
    pub monitor_changed: bool,
    pub pet_x: f32,
    pub pet_y: f32,
}

impl UserActivitySnapshot {
    pub fn cursor_dir_from_pet(&self) -> i8 {
        if self.cursor_x >= self.pet_x {
            1
        } else {
            -1
        }
    }
}

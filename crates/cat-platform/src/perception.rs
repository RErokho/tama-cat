//! Win32 sensors → cat_core::UserActivitySnapshot (no secrets / no clipboard text).

use cat_core::{DisplayInfo, UserActivitySnapshot};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_ESCAPE,
    VK_LBUTTON, VK_LWIN, VK_RBUTTON, VK_RWIN, VK_RETURN, VK_S, VK_SNAPSHOT, VK_VOLUME_DOWN,
    VK_VOLUME_UP, VK_Z,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

pub struct PerceptionSensor {
    last_cursor: Option<(i32, i32)>,
    idle_secs: f32,
    was_idle: bool,
    last_fg: HWND,
    last_window_count: usize,
    last_clip_seq: u32,
    last_caps: bool,
    key_hits: f32,
    backspace_hits: f32,
    sample_accum: f32,
    last_hour: u8,
    enter_edge: bool,
    esc_edge: bool,
    ctrl_s_edge: bool,
    ctrl_z_edge: bool,
    caps_edge: bool,
    snapshot_edge: bool,
    unlocked_pulse: bool,
    locked: bool,
    deep_idle_secs: f32,
    last_monitor_id: Option<u32>,
    vol_up_edge: bool,
    vol_down_edge: bool,
    arrow_l_edge: bool,
    arrow_r_edge: bool,
}

impl Default for PerceptionSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionSensor {
    pub fn new() -> Self {
        Self {
            last_cursor: None,
            idle_secs: 0.0,
            was_idle: false,
            last_fg: HWND::default(),
            last_window_count: 0,
            last_clip_seq: 0,
            last_caps: false,
            key_hits: 0.0,
            backspace_hits: 0.0,
            sample_accum: 0.0,
            last_hour: 12,
            enter_edge: false,
            esc_edge: false,
            ctrl_s_edge: false,
            ctrl_z_edge: false,
            caps_edge: false,
            snapshot_edge: false,
            unlocked_pulse: false,
            locked: false,
            deep_idle_secs: 0.0,
            last_monitor_id: None,
            vol_up_edge: false,
            vol_down_edge: false,
            arrow_l_edge: false,
            arrow_r_edge: false,
        }
    }

    /// Soft signal from session notifications (optional).
    pub fn set_session_locked(&mut self, locked: bool) {
        if self.locked && !locked {
            self.unlocked_pulse = true;
        }
        self.locked = locked;
    }

    pub fn sample(
        &mut self,
        dt: f32,
        displays: &[DisplayInfo],
        window_count: usize,
        left_double_click_pet: bool,
    ) -> UserActivitySnapshot {
        let (cx, cy) = cursor_pos();
        let (dx, dy, speed) = if let Some((lx, ly)) = self.last_cursor {
            let dx = (cx - lx) as f32;
            let dy = (cy - ly) as f32;
            let speed = if dt > 0.0001 {
                (dx * dx + dy * dy).sqrt() / dt
            } else {
                0.0
            };
            (dx, dy, speed)
        } else {
            (0.0, 0.0, 0.0)
        };
        self.last_cursor = Some((cx, cy));

        self.sample_accum += dt;
        let keys_down = poll_typing_activity();
        let long_away = self.deep_idle_secs > 120.0;
        if keys_down > 0 {
            self.key_hits += keys_down as f32;
            self.idle_secs = 0.0;
            self.deep_idle_secs = 0.0;
        }
        if speed > 25.0 {
            self.idle_secs = 0.0;
            self.deep_idle_secs = 0.0;
        } else if keys_down == 0 {
            self.idle_secs += dt;
            self.deep_idle_secs += dt;
        }

        // Session lock heuristic: no foreground / desktop switch while idle.
        let fg_check = unsafe { GetForegroundWindow() };
        if fg_check.is_invalid() && self.idle_secs > 2.0 {
            if !self.locked {
                self.locked = true;
            }
        } else if self.locked && !fg_check.is_invalid() && (keys_down > 0 || speed > 40.0) {
            self.locked = false;
            self.unlocked_pulse = true;
        }

        let enter = edge_key(VK_RETURN, &mut self.enter_edge);
        let esc = edge_key(VK_ESCAPE, &mut self.esc_edge);
        let backspace = key_down(VK_BACK);
        if backspace {
            self.backspace_hits += 1.0;
            self.idle_secs = 0.0;
        }

        let ctrl = key_down(VK_CONTROL);
        let ctrl_s = ctrl && edge_key(VK_S, &mut self.ctrl_s_edge);
        let ctrl_z = ctrl && edge_key(VK_Z, &mut self.ctrl_z_edge);
        let caps_now = unsafe { GetKeyState(VK_CAPITAL.0 as i32) & 1 } != 0;
        if caps_now != self.last_caps {
            self.caps_edge = true;
            self.last_caps = caps_now;
        }
        let caps_toggled = self.caps_edge;
        self.caps_edge = false;

        let print_screen = edge_key(VK_SNAPSHOT, &mut self.snapshot_edge);
        let right_mouse = key_down(VK_RBUTTON);
        let left_mouse = key_down(VK_LBUTTON);

        let fg = unsafe { GetForegroundWindow() };
        let foreground_changed = fg != self.last_fg && !fg.is_invalid();
        if !fg.is_invalid() {
            self.last_fg = fg;
        }

        let window_opened = window_count > self.last_window_count;
        let window_closed = window_count < self.last_window_count;
        self.last_window_count = window_count;

        let clip_seq = clipboard_seq();
        let clipboard_changed = clip_seq != 0 && clip_seq != self.last_clip_seq && self.last_clip_seq != 0;
        if clip_seq != 0 {
            self.last_clip_seq = clip_seq;
        }

        let mut typing_rate = 0.0;
        let backspace_burst = self.backspace_hits >= 4.0;
        if self.sample_accum >= 0.5 {
            typing_rate = self.key_hits / self.sample_accum;
            self.key_hits = 0.0;
            if !backspace_burst {
                self.backspace_hits *= 0.35;
            } else {
                self.backspace_hits = 0.0;
            }
            self.sample_accum = 0.0;
        }
        let typing = typing_rate > 0.5 || keys_down > 0;

        let returned_from_idle =
            self.was_idle && self.idle_secs < 1.0 && (typing || speed > 40.0);
        self.was_idle = self.idle_secs > 20.0;

        let (vx, vy, vw, vh) = virtual_screen();
        let cursor_offscreen =
            cx < vx - 2 || cy < vy - 2 || cx > vx + vw + 2 || cy > vy + vh + 2;
        let cursor_in_corner = near_corner(cx, cy, displays);

        let selection_likely = left_mouse && speed > 15.0 && speed < 600.0 && !typing;
        let selection_large = selection_likely && speed > 80.0 && dx.abs() + dy.abs() > 80.0;
        let file_drag_likely = left_mouse && speed > 200.0 && !typing;

        let scroll_delta = if dy.abs() > dx.abs() && speed > 180.0 && speed < 2500.0 && !left_mouse {
            dy.signum() * (speed / 400.0).min(3.0)
        } else {
            0.0
        };

        let hour = system_hour();
        let dark_theme_likely = hour >= 20 || hour < 7;
        self.last_hour = hour;

        let session_unlocked = self.unlocked_pulse || (returned_from_idle && long_away);
        self.unlocked_pulse = false;

        let volume_changed = edge_key(VK_VOLUME_UP, &mut self.vol_up_edge)
            || edge_key(VK_VOLUME_DOWN, &mut self.vol_down_edge);
        // Win+Ctrl+Left/Right proxy: Win held + left/right arrow edge.
        let win = key_down(VK_LWIN) || key_down(VK_RWIN);
        let arrow_l = edge_key(VIRTUAL_KEY(0x25), &mut self.arrow_l_edge);
        let arrow_r = edge_key(VIRTUAL_KEY(0x27), &mut self.arrow_r_edge);
        let virtual_desktop_changed = win && ctrl && (arrow_l || arrow_r);

        let monitor_id = display_at(cx as f32, cy as f32, displays);
        let monitor_changed = matches!((self.last_monitor_id, monitor_id), (Some(a), Some(b)) if a != b);
        if monitor_id.is_some() {
            self.last_monitor_id = monitor_id;
        }

        // Coarse heuristics (no UI automation / no pixel OCR).
        let toast_likely = foreground_changed && window_opened;
        let error_ui_likely = activity_error_proxy(foreground_changed, window_opened, esc);
        let loading_likely = !typing && left_mouse && speed < 8.0 && self.idle_secs > 1.5;

        UserActivitySnapshot {
            cursor_x: cx as f32,
            cursor_y: cy as f32,
            cursor_dx: dx,
            cursor_dy: dy,
            cursor_speed: speed,
            typing_rate,
            typing,
            enter_pressed: enter,
            backspace_burst,
            caps_lock_toggled: caps_toggled,
            ctrl_s,
            ctrl_z,
            esc_pressed: esc,
            scroll_delta,
            idle_secs: self.idle_secs,
            returned_from_idle,
            foreground_changed,
            window_opened,
            window_closed,
            session_locked: self.locked,
            session_unlocked,
            virtual_desktop_changed,
            volume_changed,
            clipboard_changed,
            selection_likely,
            selection_large,
            cursor_offscreen,
            cursor_near_pet: false, // filled by PetCore
            cursor_in_corner,
            print_screen,
            right_mouse,
            left_double_click_pet,
            file_drag_likely,
            hour_of_day: hour,
            dark_theme_likely,
            toast_likely,
            error_ui_likely,
            loading_likely,
            monitor_changed,
            pet_x: 0.0,
            pet_y: 0.0,
        }
    }
}

fn cursor_pos() -> (i32, i32) {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        (pt.x, pt.y)
    }
}

fn key_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000 != 0 }
}

fn edge_key(vk: VIRTUAL_KEY, was_down: &mut bool) -> bool {
    let down = key_down(vk);
    let edge = down && !*was_down;
    *was_down = down;
    edge
}

fn poll_typing_activity() -> u32 {
    // Sample printable / navigation cluster without logging which keys.
    static KEYS: &[VIRTUAL_KEY] = &[
        VIRTUAL_KEY(0x20), // space
        VIRTUAL_KEY(0x30),
        VIRTUAL_KEY(0x31),
        VIRTUAL_KEY(0x32),
        VIRTUAL_KEY(0x33),
        VIRTUAL_KEY(0x34),
        VIRTUAL_KEY(0x35),
        VIRTUAL_KEY(0x36),
        VIRTUAL_KEY(0x37),
        VIRTUAL_KEY(0x38),
        VIRTUAL_KEY(0x39),
        VIRTUAL_KEY(0x41),
        VIRTUAL_KEY(0x42),
        VIRTUAL_KEY(0x43),
        VIRTUAL_KEY(0x44),
        VIRTUAL_KEY(0x45),
        VIRTUAL_KEY(0x46),
        VIRTUAL_KEY(0x47),
        VIRTUAL_KEY(0x48),
        VIRTUAL_KEY(0x49),
        VIRTUAL_KEY(0x4A),
        VIRTUAL_KEY(0x4B),
        VIRTUAL_KEY(0x4C),
        VIRTUAL_KEY(0x4D),
        VIRTUAL_KEY(0x4E),
        VIRTUAL_KEY(0x4F),
        VIRTUAL_KEY(0x50),
        VIRTUAL_KEY(0x51),
        VIRTUAL_KEY(0x52),
        VIRTUAL_KEY(0x53),
        VIRTUAL_KEY(0x54),
        VIRTUAL_KEY(0x55),
        VIRTUAL_KEY(0x56),
        VIRTUAL_KEY(0x57),
        VIRTUAL_KEY(0x58),
        VIRTUAL_KEY(0x59),
        VIRTUAL_KEY(0x5A),
        VK_BACK,
        VK_RETURN,
    ];
    let mut n = 0u32;
    for vk in KEYS {
        if key_down(*vk) {
            n += 1;
        }
    }
    n
}

fn clipboard_seq() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

fn virtual_screen() -> (i32, i32, i32, i32) {
    unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

fn near_corner(cx: i32, cy: i32, displays: &[DisplayInfo]) -> bool {
    const PAD: f32 = 28.0;
    for d in displays {
        let near_x = (cx as f32 - d.left).abs() < PAD || (d.right - cx as f32).abs() < PAD;
        let near_y = (cy as f32 - d.top).abs() < PAD || (d.bottom - cy as f32).abs() < PAD;
        if near_x && near_y {
            return true;
        }
    }
    false
}

fn system_hour() -> u8 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Local offset approx: use UTC hour; good enough for night mood heuristic.
    ((secs / 3600) % 24) as u8
}

fn display_at(x: f32, y: f32, displays: &[DisplayInfo]) -> Option<u32> {
    displays
        .iter()
        .find(|d| x >= d.left && x < d.right && y >= d.top && y < d.bottom)
        .map(|d| d.id)
}

fn activity_error_proxy(fg_changed: bool, window_opened: bool, esc: bool) -> bool {
    // Heuristic stand-in until UI Automation: rapid dialog churn + Esc.
    fg_changed && window_opened && esc
}

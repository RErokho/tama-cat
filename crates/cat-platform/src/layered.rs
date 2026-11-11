//! Layered always-on-top pet window with mouse capture and alpha hit-test.

use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW,
    GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassExW, SetTimer, SetWindowLongPtrW,
    SetWindowPos, TranslateMessage, UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWLP_USERDATA, HWND_TOPMOST, HTCLIENT, HTTRANSPARENT, IDC_ARROW, MSG, SWP_NOACTIVATE,
    SWP_SHOWWINDOW, ULW_ALPHA, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_NCHITTEST, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("windows: {0}")]
    Win(#[from] windows::core::Error),
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Copy)]
pub enum WindowEvent {
    Quit,
    LeftDown { x: i32, y: i32 },
    LeftUp { x: i32, y: i32 },
    MouseMove { x: i32, y: i32, dragging: bool },
    RightUp,
    Tick,
}

pub struct PetWindow {
    hwnd: HWND,
    dragging: bool,
}

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);

struct HwndState {
    queue: VecDeque<Pending>,
    /// Last presented frame alpha (A channel only), row-major, for HTTRANSPARENT.
    hit_w: u32,
    hit_h: u32,
    hit_alpha: Vec<u8>,
    win_x: i32,
    win_y: i32,
}

#[derive(Clone, Copy)]
enum Pending {
    LeftDown { x: i32, y: i32 },
    LeftUp { x: i32, y: i32 },
    Move { x: i32, y: i32 },
    RightUp,
    Tick,
    Destroy,
}

impl PetWindow {
    pub fn create() -> Result<Self, PlatformError> {
        unsafe {
            let hinstance = GetModuleHandleW(None)?;
            if !CLASS_REGISTERED.swap(true, Ordering::SeqCst) {
                let class = WNDCLASSEXW {
                    cbSize: size_of::<WNDCLASSEXW>() as u32,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(wnd_proc),
                    hInstance: hinstance.into(),
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    lpszClassName: w!("DesktopCatRustWindow"),
                    ..Default::default()
                };
                if RegisterClassExW(&class) == 0 {
                    return Err(PlatformError::Msg("RegisterClassExW failed".into()));
                }
            }

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                w!("DesktopCatRustWindow"),
                w!("Desktop Cat"),
                WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                200,
                128,
                None,
                None,
                hinstance,
                None,
            )?;

            // Own state before first messages.
            let state = Box::new(HwndState {
                queue: VecDeque::new(),
                hit_w: 0,
                hit_h: 0,
                hit_alpha: Vec::new(),
                win_x: 200,
                win_y: 200,
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                200,
                200,
                200,
                128,
                SWP_SHOWWINDOW | SWP_NOACTIVATE,
            )?;

            let _ = SetTimer(hwnd, 1, 16, None);

            Ok(Self {
                hwnd,
                dragging: false,
            })
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn present_bgra(
        &self,
        bgra: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<(), PlatformError> {
        unsafe {
            // Keep hit-mask in sync with what is on screen.
            if let Some(state) = state_mut(self.hwnd) {
                state.win_x = x;
                state.win_y = y;
                state.hit_w = width;
                state.hit_h = height;
                let px = (width * height) as usize;
                state.hit_alpha.resize(px, 0);
                for i in 0..px {
                    state.hit_alpha[i] = bgra[i * 4 + 3];
                }
            }

            let hdc_screen = GetDC(None);
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0 as u32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbmp = CreateDIBSection(hdc_mem, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
            if !bits.is_null() && bgra.len() >= (width * height * 4) as usize {
                std::ptr::copy_nonoverlapping(
                    bgra.as_ptr(),
                    bits as *mut u8,
                    (width * height * 4) as usize,
                );
            }
            let old = SelectObject(hdc_mem, hbmp);
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let mut pt_dst = POINT { x, y };
            let size = SIZE {
                cx: width as i32,
                cy: height as i32,
            };
            let mut pt_src = POINT { x: 0, y: 0 };
            UpdateLayeredWindow(
                self.hwnd,
                hdc_screen,
                Some(&mut pt_dst),
                Some(&size),
                hdc_mem,
                Some(&mut pt_src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )?;
            SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
        }
        Ok(())
    }

    pub fn run_blocking_until_quit<F: FnMut(&mut Self, WindowEvent)>(&mut self, mut on_event: F) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                // Drain full queue after each message so clicks aren't lost to Tick.
                while let Some(ev) = pop_pending(self.hwnd) {
                    let mapped = match ev {
                        Pending::LeftDown { x, y } => {
                            self.dragging = true;
                            let _ = SetCapture(self.hwnd);
                            WindowEvent::LeftDown { x, y }
                        }
                        Pending::LeftUp { x, y } => {
                            self.dragging = false;
                            let _ = ReleaseCapture();
                            WindowEvent::LeftUp { x, y }
                        }
                        Pending::Move { x, y } => WindowEvent::MouseMove {
                            x,
                            y,
                            dragging: self.dragging,
                        },
                        Pending::RightUp => WindowEvent::RightUp,
                        Pending::Tick => WindowEvent::Tick,
                        Pending::Destroy => WindowEvent::Quit,
                    };
                    on_event(self, mapped);
                    if matches!(mapped, WindowEvent::Quit) {
                        return;
                    }
                }
            }
        }
    }
}

impl Drop for PetWindow {
    fn drop(&mut self) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA);
            if ptr != 0 {
                let _ = Box::from_raw(ptr as *mut HwndState);
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            }
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn state_mut(hwnd: HWND) -> Option<&'static mut HwndState> {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if ptr == 0 {
            None
        } else {
            Some(&mut *(ptr as *mut HwndState))
        }
    }
}

fn push_pending(hwnd: HWND, ev: Pending) {
    if let Some(state) = state_mut(hwnd) {
        // Cap queue so a stuck loop can't grow forever; keep newest input.
        if state.queue.len() > 64 {
            state.queue.pop_front();
        }
        // Coalesce consecutive moves / ticks.
        match (state.queue.back(), ev) {
            (Some(Pending::Move { .. }), Pending::Move { .. }) => {
                *state.queue.back_mut().unwrap() = ev;
            }
            (Some(Pending::Tick), Pending::Tick) => {}
            _ => state.queue.push_back(ev),
        }
    }
}

fn pop_pending(hwnd: HWND) -> Option<Pending> {
    state_mut(hwnd)?.queue.pop_front()
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            // Per-pixel: transparent areas click through to desktop.
            let (cx, cy) = cursor();
            if let Some(state) = state_mut(hwnd) {
                if state.hit_w > 0 && state.hit_h > 0 {
                    let lx = cx - state.win_x;
                    let ly = cy - state.win_y;
                    if lx >= 0
                        && ly >= 0
                        && (lx as u32) < state.hit_w
                        && (ly as u32) < state.hit_h
                    {
                        let idx = (ly as u32 * state.hit_w + lx as u32) as usize;
                        if state.hit_alpha.get(idx).copied().unwrap_or(0) < 16 {
                            return LRESULT(HTTRANSPARENT as isize);
                        }
                        return LRESULT(HTCLIENT as isize);
                    }
                }
            }
            return LRESULT(HTTRANSPARENT as isize);
        }
        WM_LBUTTONDOWN => {
            let (x, y) = cursor();
            push_pending(hwnd, Pending::LeftDown { x, y });
        }
        WM_LBUTTONUP => {
            let (x, y) = cursor();
            push_pending(hwnd, Pending::LeftUp { x, y });
        }
        WM_MOUSEMOVE => {
            let (x, y) = cursor();
            push_pending(hwnd, Pending::Move { x, y });
        }
        WM_RBUTTONUP => {
            push_pending(hwnd, Pending::RightUp);
        }
        WM_TIMER => {
            push_pending(hwnd, Pending::Tick);
        }
        WM_DESTROY => {
            push_pending(hwnd, Pending::Destroy);
            PostQuitMessage(0);
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn cursor() -> (i32, i32) {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        (pt.x, pt.y)
    }
}

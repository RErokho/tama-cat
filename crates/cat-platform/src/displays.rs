use cat_core::DisplayInfo;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

const MONITORINFOF_PRIMARY: u32 = 1;

struct Collect(Vec<DisplayInfo>);

pub fn list_displays() -> Vec<DisplayInfo> {
    let mut collect = Collect(Vec::new());
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum),
            LPARAM(&mut collect as *mut _ as isize),
        );
    }
    if collect.0.is_empty() {
        let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) } as f32;
        let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) } as f32;
        let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) } as f32;
        let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) } as f32;
        collect.0.push(DisplayInfo {
            id: 0,
            left: vx,
            right: vx + vw,
            top: vy,
            bottom: vy + vh,
            floor_y: vy + vh - 2.0,
            work_top: vy,
        });
    }
    collect.0.sort_by_key(|d| if d.id == 0 { 0 } else { 1 });
    collect.0
}

unsafe extern "system" fn monitor_enum(
    monitor: windows::Win32::Graphics::Gdi::HMONITOR,
    _hdc: windows::Win32::Graphics::Gdi::HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let collect = &mut *(data.0 as *mut Collect);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(monitor, &mut info).as_bool() {
        let r = info.rcMonitor;
        let w = info.rcWork;
        let id = if (info.dwFlags & MONITORINFOF_PRIMARY) != 0 {
            0
        } else {
            (collect.0.len() + 1) as u32
        };
        collect.0.push(DisplayInfo {
            id,
            left: r.left as f32,
            right: r.right as f32,
            top: r.top as f32,
            bottom: r.bottom as f32,
            floor_y: w.bottom as f32 - 2.0,
            work_top: w.top as f32,
        });
    }
    TRUE
}

#[allow(dead_code)]
pub fn primary_display() -> Option<DisplayInfo> {
    unsafe {
        let mon = MonitorFromPoint(
            windows::Win32::Foundation::POINT { x: 0, y: 0 },
            MONITOR_DEFAULTTOPRIMARY,
        );
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(mon, &mut info).as_bool() {
            return None;
        }
        let r = info.rcMonitor;
        let w = info.rcWork;
        Some(DisplayInfo {
            id: 0,
            left: r.left as f32,
            right: r.right as f32,
            top: r.top as f32,
            bottom: r.bottom as f32,
            floor_y: w.bottom as f32 - 2.0,
            work_top: w.top as f32,
        })
    }
}

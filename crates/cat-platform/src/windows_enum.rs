use cat_core::{Surface, SurfaceKind};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW, IsIconic, IsWindowVisible,
    GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

const MIN_WIDTH: i32 = 120;
const MIN_HEIGHT: i32 = 80;
const TOP_BAND: f32 = 28.0;

#[derive(Clone, Copy)]
struct WinRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

struct Bucket(Vec<WinRect>);

pub fn list_window_surfaces() -> Vec<Surface> {
    let mut bucket = Bucket(Vec::new());
    unsafe {
        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(&mut bucket as *mut _ as isize),
        );
    }
    let wins = bucket.0;
    let mut results = Vec::new();
    let mut index = 0usize;

    for (i, win) in wins.iter().enumerate() {
        let mut segs = vec![(win.left as f32, (win.right - win.left) as f32)];
        for above in wins.iter().take(i) {
            if above.bottom as f32 <= win.top as f32
                || above.top as f32 >= win.top as f32 + TOP_BAND
            {
                continue;
            }
            segs = subtract_segs(
                &segs,
                above.left as f32,
                (above.right - above.left) as f32,
            );
            if segs.is_empty() {
                break;
            }
        }
        if segs.is_empty() {
            continue;
        }
        for (x, w) in segs {
            results.push(Surface {
                id: format!("window-top-{index}"),
                kind: SurfaceKind::WindowTop,
                x,
                y: win.top as f32,
                width: w,
                height: TOP_BAND,
            });
        }
        let h = (win.bottom - win.top) as f32;
        results.push(Surface {
            id: format!("window-side-L-{index}"),
            kind: SurfaceKind::WindowSide,
            x: win.left as f32 - 6.0,
            y: win.top as f32,
            width: 12.0,
            height: h,
        });
        results.push(Surface {
            id: format!("window-side-R-{index}"),
            kind: SurfaceKind::WindowSide,
            x: win.right as f32 - 6.0,
            y: win.top as f32,
            width: 12.0,
            height: h,
        });
        index += 1;
    }
    results
}

fn subtract_segs(segs: &[(f32, f32)], cut_x: f32, cut_w: f32) -> Vec<(f32, f32)> {
    let cut_l = cut_x;
    let cut_r = cut_x + cut_w;
    let mut out = Vec::new();
    for &(x, w) in segs {
        let l = x;
        let r = x + w;
        if cut_r <= l || cut_l >= r {
            out.push((x, w));
            continue;
        }
        if cut_l > l {
            out.push((l, cut_l - l));
        }
        if cut_r < r {
            out.push((cut_r, r - cut_r));
        }
    }
    out.into_iter().filter(|(_, w)| *w >= 36.0).collect()
}

unsafe extern "system" fn enum_proc(hwnd: HWND, data: LPARAM) -> BOOL {
    let bucket = &mut *(data.0 as *mut Bucket);
    if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
        return TRUE;
    }
    if GetWindowTextLengthW(hwnd) <= 0 {
        return TRUE;
    }
    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    if (ex & WS_EX_TOOLWINDOW.0 != 0) && (ex & WS_EX_APPWINDOW.0 == 0) {
        return TRUE;
    }
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return TRUE;
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width < MIN_WIDTH || height < MIN_HEIGHT || rect.top < -10_000 {
        return TRUE;
    }
    bucket.0.push(WinRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    });
    TRUE
}

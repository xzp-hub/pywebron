#[cfg(target_os = "windows")]
use crate::configs::WindowCornerPreference;
use image::open;
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use tao::window::Icon;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWM_WINDOW_CORNER_PREFERENCE,
    },
    UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GetPropW, GetWindowLongPtrW, SetPropW, SetWindowLongPtrW,
        SetWindowPos, GWLP_WNDPROC, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        SWP_SHOWWINDOW, WM_NCCALCSIZE, WNDPROC,
    },
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "windows")]
unsafe extern "system" fn frameless_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let prop_name = windows::core::w!("pywebron_orig_proc");
    let orig_proc_ptr = GetPropW(hwnd, prop_name);
    let orig_proc: WNDPROC = std::mem::transmute(orig_proc_ptr.0);

    if msg == WM_NCCALCSIZE {
        // 无论 wparam 是 TRUE 还是 FALSE，返回 0 都表示客户区覆盖整个窗口
        // 这会隐藏 WS_THICKFRAME 的可视边框，但保留系统的拖拽缩放功能
        return LRESULT(0);
    }

    if let Some(proc) = orig_proc {
        CallWindowProcW(Some(proc), hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

/// 将窗口子类化，拦截 WM_NCCALCSIZE 以移除可视边框但保留缩放功能
#[cfg(target_os = "windows")]
pub fn make_window_frameless_but_resizable(hwnd: HWND) {
    unsafe {
        let prop_name = windows::core::w!("pywebron_orig_proc");
        // 避免重复子类化
        if !GetPropW(hwnd, prop_name).0.is_null() {
            return;
        }

        // 添加 WS_THICKFRAME 以允许系统缩放
        use windows::Win32::UI::WindowsAndMessaging::{GWL_STYLE, WS_THICKFRAME};
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, style | WS_THICKFRAME.0 as isize);

        let orig_proc = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, frameless_proc as *const () as usize as isize);
        if orig_proc != 0 {
            let _ = SetPropW(hwnd, prop_name, windows::Win32::Foundation::HANDLE(orig_proc as *mut std::ffi::c_void).into());
        }

        // 刷新窗口框架
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );
    }
}

pub fn generate_window_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    (timestamp % 1000000000000) * 1000 + (counter % 1000)
}

pub fn generate_win_icon(path: String) -> Option<Icon> {
    let p = &path;
    if p.is_empty() || !Path::new(p).exists() {
        return None;
    }
    let img = open(p).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn set_window_corner(hwnd: HWND, pref: WindowCornerPreference) -> Result<(), String> {
    unsafe {
        let val = pref.to_dwm();
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &val as *const _ as *const _,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        )
        .map(|_| {
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        })
        .map_err(|e| format!("{:?}", e))
    }
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn set_window_corner_with_retry(hwnd: HWND, pref: WindowCornerPreference, retries: u32) {
    for _ in 0..retries {
        if set_window_corner(hwnd, pref).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

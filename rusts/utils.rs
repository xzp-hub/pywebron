#[cfg(target_os = "windows")]
use crate::configs::WindowCorners;
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
    Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn},
    UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GetClientRect, GetPropW, GetWindowLongPtrW,
        SetProcessDPIAware, SetPropW, SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WM_NCCALCSIZE,
        WNDPROC,
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
        return LRESULT(0);
    }

    if let Some(proc) = orig_proc {
        CallWindowProcW(Some(proc), hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

#[cfg(target_os = "windows")]
pub fn make_window_frameless_but_resizable(hwnd: HWND) {
    unsafe {
        let prop_name = windows::core::w!("pywebron_orig_proc");
        if !GetPropW(hwnd, prop_name).0.is_null() {
            return;
        }

        use windows::Win32::UI::WindowsAndMessaging::{GWL_STYLE, WS_THICKFRAME};
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, style | WS_THICKFRAME.0 as isize);

        let orig_proc = SetWindowLongPtrW(
            hwnd,
            GWLP_WNDPROC,
            frameless_proc as *const () as usize as isize,
        );
        if orig_proc != 0 {
            let _ = SetPropW(
                hwnd,
                prop_name,
                windows::Win32::Foundation::HANDLE(orig_proc as *mut std::ffi::c_void).into(),
            );
        }

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
pub fn set_window_corner(hwnd: HWND, pref: WindowCorners) -> Result<(), String> {
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
pub fn set_window_corner_with_retry(hwnd: HWND, pref: WindowCorners, retries: u32) {
    for _ in 0..retries {
        if set_window_corner(hwnd, pref).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// 设置窗口圆角区域（通过裁剪窗口区域实现真正的圆角）
#[cfg(target_os = "windows")]
pub fn set_window_rounded_region(hwnd: HWND, radius: u32) -> Result<(), String> {
    unsafe {
        let mut rect = windows::Win32::Foundation::RECT::default();
        if GetClientRect(hwnd, &mut rect).is_err() {
            return Err("Failed to get client rect".to_string());
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // 创建圆角矩形区域
        let hrgn = CreateRoundRectRgn(0, 0, width, height, radius as i32, radius as i32);
        
        if hrgn.is_invalid() {
            return Err("Failed to create rounded region".to_string());
        }

        // 设置窗口区域（第二个参数需要 Option<HRGN>，第三个参数 true 表示重绘窗口）
        let result = SetWindowRgn(hwnd, Some(hrgn), true);
        
        if result == 0 {
            return Err("Failed to set window region".to_string());
        }

        eprintln!("[Window] 窗口圆角区域设置成功: radius={}px, size={}x{}", radius, width, height);
        Ok(())
    }
}

/// 设置进程 DPI 意识（Windows）
#[cfg(target_os = "windows")]
pub fn setup_dpi_awareness() {
    unsafe {
        let shcore = b"shcore.dll\0";
        let hmodule = windows::Win32::System::LibraryLoader::GetModuleHandleA(
            windows::core::PCSTR(shcore.as_ptr()),
        );

        if let Ok(hmodule) = hmodule {
            let proc_name = b"SetProcessDpiAwareness\0";
            let proc_addr = windows::Win32::System::LibraryLoader::GetProcAddress(
                hmodule,
                windows::core::PCSTR(proc_name.as_ptr()),
            );

            if let Some(addr) = proc_addr {
                type SetProcessDpiAwarenessFn = unsafe extern "system" fn(u32) -> i32;
                let func: SetProcessDpiAwarenessFn = std::mem::transmute(addr);
                let result = func(2);

                if result == 0 {
                    return;
                }
            }
        }

        let _ = SetProcessDPIAware();
    }
}

/// 设置进程 DPI 意识（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
pub fn setup_dpi_awareness() {
    std::env::set_var("GDK_SCALE", "1");
    std::env::set_var("GDK_DPI_SCALE", "1");
}

#[cfg(target_os = "windows")]
use crate::configs::WindowCorners;
use dashmap::DashMap;
use image::open;
use once_cell::sync::Lazy;
use std::path::Path;
use tao::window::Icon;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWM_WINDOW_CORNER_PREFERENCE,
    },
    UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GetPropW, GetWindowLongPtrW, SetProcessDPIAware, SetPropW,
        SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOZORDER, SWP_SHOWWINDOW, WM_NCCALCSIZE, WNDPROC,
    },
};

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

type CachedIcon = (Vec<u8>, u32, u32);
static ICON_CACHE: Lazy<DashMap<String, CachedIcon>> = Lazy::new(DashMap::new);

pub fn generate_win_icon(path: String) -> Option<Icon> {
    let p = path.trim();
    if p.is_empty() || !Path::new(p).exists() {
        return None;
    }

    if let Some(icon) = ICON_CACHE.get(p) {
        let (rgba, width, height) = icon.value();
        return Icon::from_rgba(rgba.clone(), *width, *height).ok();
    }

    let img = open(p).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.into_raw();
    let icon = Icon::from_rgba(rgba.clone(), w, h).ok()?;
    ICON_CACHE.insert(p.to_string(), (rgba, w, h));
    Some(icon)
}

#[cfg(target_os = "windows")]
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

/// Set process DPI awareness (Windows)
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

/// Set process DPI awareness (non-Windows platforms)
#[cfg(not(target_os = "windows"))]
pub fn setup_dpi_awareness() {
    std::env::set_var("GDK_SCALE", "1");
    std::env::set_var("GDK_DPI_SCALE", "1");
}

#[cfg(target_os = "windows")]
use crate::configs::WindowCorners;
use dashmap::DashMap;
use image::open;
use std::path::Path;
use std::sync::LazyLock;
use tao::window::Icon;

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
unsafe extern "system" {
    fn SetCurrentProcessExplicitAppUserModelID(app_id: windows::core::PCWSTR)
    -> windows::core::HRESULT;
}

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWM_WINDOW_CORNER_PREFERENCE,
    },
    UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GetPropW, GetSystemMetrics, GetWindowLongPtrW,
        GetWindowRect, IsZoomed, SetProcessDPIAware, SetPropW, SetWindowLongPtrW, SetWindowPos,
        GWLP_WNDPROC, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT, HTLEFT,
        HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, SM_CXFRAME, SM_CXPADDEDBORDER, SM_CYFRAME,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WM_NCCALCSIZE,
        WM_NCHITTEST, WM_SYSCOMMAND, WNDPROC,
    },
};

#[cfg(target_os = "windows")]
fn get_x_lparam(value: LPARAM) -> i32 {
    (value.0 as i16) as i32
}

#[cfg(target_os = "windows")]
fn get_y_lparam(value: LPARAM) -> i32 {
    ((value.0 >> 16) as i16) as i32
}

#[cfg(target_os = "windows")]
unsafe fn hit_test_resize_border(hwnd: HWND, lparam: LPARAM) -> Option<LRESULT> {
    if IsZoomed(hwnd).as_bool() {
        return None;
    }

    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return None;
    }

    let border_x = (GetSystemMetrics(SM_CXFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER)).max(8);
    let border_y = (GetSystemMetrics(SM_CYFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER)).max(8);

    let cursor = POINT {
        x: get_x_lparam(lparam),
        y: get_y_lparam(lparam),
    };

    let on_left = cursor.x >= rect.left && cursor.x < rect.left + border_x;
    let on_right = cursor.x <= rect.right && cursor.x > rect.right - border_x;
    let on_top = cursor.y >= rect.top && cursor.y < rect.top + border_y;
    let on_bottom = cursor.y <= rect.bottom && cursor.y > rect.bottom - border_y;

    let hit = match (on_left, on_right, on_top, on_bottom) {
        (true, _, true, _) => HTTOPLEFT,
        (false, true, true, _) => HTTOPRIGHT,
        (true, _, false, true) => HTBOTTOMLEFT,
        (false, true, false, true) => HTBOTTOMRIGHT,
        (true, _, _, _) => HTLEFT,
        (false, true, _, _) => HTRIGHT,
        (_, _, true, _) => HTTOP,
        (_, _, false, true) => HTBOTTOM,
        _ => return None,
    };

    Some(LRESULT(hit as isize))
}

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

    if msg == WM_NCHITTEST {
        if let Some(hit) = hit_test_resize_border(hwnd, lparam) {
            return hit;
        }

        let result = if let Some(proc) = orig_proc {
            CallWindowProcW(Some(proc), hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        };

        if result == LRESULT(HTCLIENT as isize) {
            return LRESULT(HTCAPTION as isize);
        }
        return result;
    }

    if msg == WM_SYSCOMMAND {
        let cmd = wparam.0 as u32 & 0xFFF0;
        if cmd == 0xF000 {
            // SC_SIZE — let it pass through for native resize
        }
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
        let new_style = style | WS_THICKFRAME.0 as isize;
        let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);

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
static ICON_CACHE: LazyLock<DashMap<String, CachedIcon>> = LazyLock::new(DashMap::new);

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

#[cfg(target_os = "windows")]
pub fn setup_app_user_model_id() {
    let app_id: Vec<u16> = "PyWebron.App\0".encode_utf16().collect();
    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(windows::core::PCWSTR(app_id.as_ptr()));
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

#[cfg(not(target_os = "windows"))]
pub fn setup_app_user_model_id() {}

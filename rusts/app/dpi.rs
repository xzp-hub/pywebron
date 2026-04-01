#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

/// 设置进程 DPI 意识（Windows）
///
/// 尝试使用新版 API (SetProcessDpiAwareness) 设置 PROCESS_PER_MONITOR_DPI_AWARE
/// 如果失败（旧版 Windows），则使用旧版 API (SetProcessDPIAware)
#[cfg(target_os = "windows")]
pub fn setup_dpi_awareness() {
    unsafe {
        // 尝试使用 shcore.dll 中的 SetProcessDpiAwareness
        let shcore = b"shcore.dll\0";
        let hmodule = windows::Win32::System::LibraryLoader::GetModuleHandleA(
            windows::core::PCSTR(shcore.as_ptr()),
        );

        if let Ok(hmodule) = hmodule {
            // 获取 SetProcessDpiAwareness 函数地址
            let proc_name = b"SetProcessDpiAwareness\0";
            let proc_addr = windows::Win32::System::LibraryLoader::GetProcAddress(
                hmodule,
                windows::core::PCSTR(proc_name.as_ptr()),
            );

            if let Some(addr) = proc_addr {
                // 调用 SetProcessDpiAwareness(2) - PROCESS_PER_MONITOR_DPI_AWARE
                type SetProcessDpiAwarenessFn = unsafe extern "system" fn(u32) -> i32;
                let func: SetProcessDpiAwarenessFn = std::mem::transmute(addr);
                let result = func(2); // PROCESS_PER_MONITOR_DPI_AWARE

                // 如果成功则返回
                if result == 0 {
                    return;
                }
            }
        }

        // 如果上述方法失败，使用旧版 API
        let _ = SetProcessDPIAware();
    }
}

#[cfg(not(target_os = "windows"))]
pub fn setup_dpi_awareness() {
    // 非 Windows 平台不需要 DPI 设置
}

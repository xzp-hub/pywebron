#[derive(Debug, Clone)]
pub enum UserEvent {
    CloseWindow(u64),
    // IPC 消息分发到 WebView（Arc<String> 避免多窗口广播时重复 clone）
    EvaluateScript {
        window_id: u64,
        script: std::sync::Arc<String>,
    },
    // 唤醒事件循环，处理待创建的窗口
    WakeUp,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy)]
pub enum WindowCorners {
    Default,
    DoNotRound,
    Round,
    RoundSmall,
}

#[cfg(target_os = "windows")]
impl WindowCorners {
    #[cfg(target_os = "windows")]
    pub fn to_dwm(&self) -> windows::Win32::Graphics::Dwm::DWM_WINDOW_CORNER_PREFERENCE {
        use windows::Win32::Graphics::Dwm::DWM_WINDOW_CORNER_PREFERENCE;
        DWM_WINDOW_CORNER_PREFERENCE(match self {
            WindowCorners::Default => 0,
            WindowCorners::DoNotRound => 1,
            WindowCorners::Round => 2,
            WindowCorners::RoundSmall => 3,
        })
    }
}

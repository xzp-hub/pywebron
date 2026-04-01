#[derive(Debug, Clone)]
pub enum UserEvent {
    CloseWindow(u64),
    #[cfg(target_os = "windows")]
    SetCorner {
        hwnd: usize,
        corner: u32,
    },
    // P1: IPC 消息分发到 WebView
    EvaluateScript {
        window_id: u64,
        script: String,
    },
    // 唤醒事件循环，处理待创建的窗口
    WakeUp,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy)]
pub enum WindowCornerPreference {
    Default,
    DoNotRound,
    Round,
    RoundSmall,
}

#[cfg(target_os = "windows")]
impl WindowCornerPreference {
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => WindowCornerPreference::DoNotRound,
            2 => WindowCornerPreference::Round,
            3 => WindowCornerPreference::RoundSmall,
            _ => WindowCornerPreference::Default,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn to_dwm(&self) -> windows::Win32::Graphics::Dwm::DWM_WINDOW_CORNER_PREFERENCE {
        use windows::Win32::Graphics::Dwm::DWM_WINDOW_CORNER_PREFERENCE;
        DWM_WINDOW_CORNER_PREFERENCE(match self {
            WindowCornerPreference::Default => 0,
            WindowCornerPreference::DoNotRound => 1,
            WindowCornerPreference::Round => 2,
            WindowCornerPreference::RoundSmall => 3,
        })
    }
}

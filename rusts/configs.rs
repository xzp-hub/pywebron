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
#[allow(dead_code)]
pub enum WindowCorners {
    Default,
    DoNotRound,
    Round,
    RoundSmall,
}

#[cfg(target_os = "windows")]
impl WindowCorners {
    #[allow(dead_code)]
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => WindowCorners::DoNotRound,
            2 => WindowCorners::Round,
            3 => WindowCorners::RoundSmall,
            _ => WindowCorners::Default,
        }
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
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

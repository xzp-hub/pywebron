use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub enum UserEvent {
    CloseWindow(u64),
    EvaluateScript {
        window_id: u64,
        script: std::sync::Arc<String>,
    },
    WakeUp,
}

pub static LOG_DEBUG: LazyLock<bool> = LazyLock::new(|| {
    matches!(
        std::env::var("PYWEBRON_LOG_LEVEL")
            .unwrap_or_else(|_| "error".to_string())
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "debug"
    )
});

#[inline]
pub fn debug_log(message: impl FnOnce() -> String) {
    if *LOG_DEBUG {
        eprintln!("{}", message());
    }
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

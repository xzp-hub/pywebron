mod configs;
mod utils;

mod app {
    pub mod dpi;
    pub mod invoke;
    pub mod stream;
    pub mod window;

    pub use invoke::get_handles;
    pub use invoke::invoke_handle;
    pub use stream::load_js_api;
    pub use stream::stream_recv;
    pub use stream::stream_send;
    pub use window::get_all_window_ids;
    pub use window::send_script_to_window;
    pub use window::*;
}

pub use app::*;

use pyo3::prelude::*;
use pyo3::types::PyModule;

// 使用 mimalloc 全局内存分配器（性能优于系统默认分配器）
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL_MIMALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[pymodule]
fn _pywebron_(pymodule: &Bound<'_, PyModule>) -> PyResult<()> {
    // 在模块加载时立即设置 DPI 意识（必须在程序启动早期执行）
    app::dpi::setup_dpi_awareness();

    pymodule.add_function(wrap_pyfunction!(app::init, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::run, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::register_window, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::minimize_window, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::maximize_window, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::reappear_window, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::shutdown_window, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::start_drag_window, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::setup_drag_region, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::get_handles, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::get_windows, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::stream_send, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::stream_recv, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::save_file_dialog, pymodule)?)?;
    pymodule.add_function(wrap_pyfunction!(app::invoke_handle, pymodule)?)?;
    Ok(())
}

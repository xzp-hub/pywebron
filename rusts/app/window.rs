use crate::app::load_js_api;
use crate::configs::UserEvent;
use crate::utils::{generate_win_icon, generate_window_id};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use pyo3::ffi::{PyEval_RestoreThread, PyEval_SaveThread};
use pyo3::prelude::*;
use pyo3::{Bound, PyResult};
use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use tao::platform::unix::EventLoopBuilderExtUnix;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "windows")]
use tao::platform::windows::WindowBuilderExtWindows; // 关键：导入 Windows 扩展 trait
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent::CloseRequested},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use wry::WebViewBuilderExtUnix;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

static WEBVIEW_CREATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static WINDOWS: Lazy<DashMap<u64, tao::window::Window>> = Lazy::new(DashMap::new);
static WINDOW_PROXIES: Lazy<DashMap<u64, WindowHandle>> = Lazy::new(DashMap::new);
static WINDOW_READY: Lazy<DashMap<u64, bool>> = Lazy::new(DashMap::new);
// 全局 EventLoopProxy 存储，用于跨线程发送消息到前端
static EVENT_PROXIES: Lazy<DashMap<u64, tao::event_loop::EventLoopProxy<UserEvent>>> =
    Lazy::new(DashMap::new);
// 主事件循环的 Proxy（用于唤醒事件循环）
static MAIN_EVENT_PROXY: once_cell::sync::Lazy<
    Mutex<Option<tao::event_loop::EventLoopProxy<UserEvent>>>,
> = once_cell::sync::Lazy::new(|| Mutex::new(None));
// 待创建的窗口队列
static PENDING_WINDOWS: Lazy<Mutex<HashMap<u64, WindowConfig>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
// 存储 webview 对象（WebView 不是 Send/Sync，用 unsafe wrapper 绕过）
struct SendSyncWebView(std::sync::Arc<Mutex<Option<wry::WebView>>>);
unsafe impl Send for SendSyncWebView {}
unsafe impl Sync for SendSyncWebView {}

static WEBVIEWS: Lazy<DashMap<u64, SendSyncWebView>> = Lazy::new(DashMap::new);

// HTML 文件缓存：避免重复读取文件
static HTML_CACHE: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);

// 资源文件缓存：用于自定义协议处理器
static RESOURCE_CACHE: Lazy<DashMap<String, Vec<u8>>> = Lazy::new(DashMap::new);

fn cleanup_window(window_id: u64) {
    WINDOWS.remove(&window_id);
    WINDOW_PROXIES.remove(&window_id);
    WINDOW_READY.remove(&window_id);
    EVENT_PROXIES.remove(&window_id);
    WEBVIEWS.remove(&window_id);
}

/// 获取文件的 MIME 类型
fn get_mime_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("json") => "application/json",
        Some("wasm") => "application/wasm",
        Some("txt") => "text/plain",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}

/// 向指定窗口发送 JavaScript 消息（通过事件循环）
pub fn send_script_to_window(window_id: u64, script: String) -> bool {
    let _t = std::time::Instant::now();
    let result = if let Some(proxy) = EVENT_PROXIES.get(&window_id) {
        proxy
            .send_event(UserEvent::EvaluateScript { window_id, script })
            .is_ok()
    } else {
        false
    };
    result
}

/// 在事件循环中创建实际窗口和 WebView
fn create_window_in_event_loop(
    config: &WindowConfig,
    window_id: u64,
    proxy: &tao::event_loop::EventLoopProxy<UserEvent>,
    event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
) {
    let t_total = std::time::Instant::now();
    eprintln!(
        "[Performance][Rust] create_window_in_event_loop 开始: {}",
        config.title
    );

    let id_clone = window_id;

    #[cfg(target_os = "windows")]
    let window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_window_icon(generate_win_icon(config.icon_path.clone()))
        .with_decorations(config.show_title_bar)
        .with_resizable(config.enable_resizable)
        .with_min_inner_size(LogicalSize::new(400u32, 300u32))
        .with_transparent(true)
        .with_undecorated_shadow(false);

    eprintln!(
        "[Window] 创建窗口：{} | show_title_bar={} | size={}x{}",
        config.title, config.show_title_bar, config.width, config.height
    );

    #[cfg(not(target_os = "windows"))]
    let window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(PhysicalSize::new(config.width, config.height))
        .with_window_icon(generate_win_icon(config.icon_path.clone()))
        .with_decorations(config.show_title_bar)
        .with_resizable(config.enable_resizable)
        .with_min_inner_size(PhysicalSize::new(400u32, 300u32))
        .with_transparent(true);

    let t_window_build = std::time::Instant::now();
    let window = match window_builder.build(event_loop) {
        Ok(w) => w,
        Err(_) => {
            eprintln!("[Error] 窗口创建失败：{}", config.title);
            return;
        }
    };
    eprintln!(
        "[Performance][Rust] 窗口构建耗时: {:?}",
        t_window_build.elapsed()
    );

    #[cfg(target_os = "windows")]
    let hwnd = {
        use tao::platform::windows::WindowExtWindows;
        let hwnd = window.hwnd() as *mut std::ffi::c_void;

        // Windows 透明背景设置
        use windows::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH};
        use windows::Win32::UI::WindowsAndMessaging::{SetClassLongPtrW, GCLP_HBRBACKGROUND};

        unsafe {
            let win_hwnd = windows::Win32::Foundation::HWND(hwnd);

            let hbrush = GetStockObject(BLACK_BRUSH);
            let _ = SetClassLongPtrW(win_hwnd, GCLP_HBRBACKGROUND, hbrush.0 as isize);

            // 如果窗口可调整大小，确保有 WS_THICKFRAME 样式
            if config.enable_resizable {
                use windows::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, GWL_STYLE, WS_THICKFRAME,
                };
                let style = GetWindowLongPtrW(win_hwnd, GWL_STYLE);
                let _ = SetWindowLongPtrW(win_hwnd, GWL_STYLE, style | WS_THICKFRAME.0 as isize);
            }
        }

        hwnd
    };

    // 创建 webview
    let t_webview_start = std::time::Instant::now();
    eprintln!("[Performance][Rust] 开始创建 WebView");
    let webview = {
        let t_lock = std::time::Instant::now();
        let _webview_lock = WEBVIEW_CREATE_LOCK.lock().unwrap();
        eprintln!(
            "[Performance][Rust] 获取 WEBVIEW_CREATE_LOCK 耗时: {:?}",
            t_lock.elapsed()
        );

        let html_content = config.html_content.clone();
        let link_content = config.link_content.clone();
        let dist_content = config.dist_content.clone();

        let is_url = link_content.is_some();
        let is_file_path = html_content.is_some();
        let is_dist = dist_content.is_some();

        let resolved_content = if let Some(ref link) = link_content {
            link.clone()
        } else if let Some(ref path) = html_content {
            path.clone()
        } else if let Some(ref dist) = dist_content {
            dist.clone()
        } else {
            String::new()
        };

        let proxy_for_handler = proxy.clone();
        let window_id_for_ipc = id_clone;
        let t_config_json = std::time::Instant::now();
        let window_config_json = serde_json::json!({
            "window_id": id_clone,
            "title": config.title,
            "width": config.width,
            "height": config.height,
            "show_title_bar": config.show_title_bar,
            "enable_resizable": config.enable_resizable,
            "enable_devtools": config.enable_devtools
        });
        eprintln!(
            "[Performance][Rust] 构建配置 JSON 耗时: {:?}",
            t_config_json.elapsed()
        );

        // 存储 dist 路径用于自定义协议处理
        let dist_path_for_protocol = if config.dist_content.is_some() {
            let dist_path = std::path::Path::new(config.dist_content.as_ref().unwrap());
            if dist_path.is_absolute() {
                dist_path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(dist_path)
            }
        } else {
            std::path::PathBuf::new()
        };

        let t_builder = std::time::Instant::now();
        let builder = WebViewBuilder::new()
            .with_devtools(config.enable_devtools)
            .with_transparent(true)
            .with_background_color((255, 255, 255, 255))
            .with_initialization_script(&format!(
                "window.pywebron={};{}",
                serde_json::to_string(&window_config_json).unwrap_or_default(),
                load_js_api()
            ))
            .with_ipc_handler(move |request| {
                handle_ipc_message(request, window_id_for_ipc, &proxy_for_handler);
            })
            .with_custom_protocol("app".to_string(), move |_id, request| {
                let t_protocol_start = std::time::Instant::now();
                let uri = request.uri().to_string();

                // Windows: http://app.<path>, 其他平台: app://<path>
                let file_path = uri
                    .strip_prefix("http://app.")
                    .or_else(|| uri.strip_prefix("https://app."))
                    .or_else(|| uri.strip_prefix("app://"))
                    .unwrap_or("")
                    .trim_end_matches('/')
                    .to_string();

                let full_path = dist_path_for_protocol.join(&file_path);
                let cache_key = full_path.to_string_lossy().to_string();

                // 先检查缓存
                if let Some(cached_data) = RESOURCE_CACHE.get(&cache_key) {
                    // 缓存命中 - 静默处理，不打印日志（性能最优）
                    let mime_type = get_mime_type(&full_path);
                    return http::Response::builder()
                        .header("Content-Type", mime_type)
                        .header("Cache-Control", "public, max-age=31536000")
                        .body(std::borrow::Cow::Owned(cached_data.clone()))
                        .unwrap();
                }

                // 缓存未命中，读取文件
                match std::fs::read(&full_path) {
                    Ok(data) => {
                        // 存入缓存
                        RESOURCE_CACHE.insert(cache_key.clone(), data.clone());

                        let mime_type = get_mime_type(&full_path);
                        let total_time = t_protocol_start.elapsed();

                        // 只在开发模式下打印详细日志（可选）
                        #[cfg(debug_assertions)]
                        if total_time.as_millis() > 5 {
                            eprintln!(
                                "[Cache] 加载资源: {} | 耗时: {:?} | 大小: {:.2} KB",
                                file_path,
                                total_time,
                                data.len() as f64 / 1024.0
                            );
                        }

                        http::Response::builder()
                            .header("Content-Type", mime_type)
                            .header("Cache-Control", "public, max-age=31536000")
                            .body(std::borrow::Cow::Owned(data))
                            .unwrap()
                    }
                    Err(e) => {
                        eprintln!("[Error][Cache] 文件读取失败: {} | 错误: {}", file_path, e);
                        http::Response::builder()
                            .status(404)
                            .body(std::borrow::Cow::Owned(b"Not Found".to_vec()))
                            .unwrap()
                    }
                }
            });
        eprintln!(
            "[Performance][Rust] 构建 WebViewBuilder 耗时: {:?}",
            t_builder.elapsed()
        );

        #[cfg(target_os = "windows")]
        let builder = builder.with_additional_browser_args(
            "--disable-features=IsolateOrigins,site-per-process,ThirdPartyStoragePartitioning,ThirdPartyCookiesBlocking --allow-file-access-from-files --disable-web-security",
        );

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        let result = {
            use gtk::prelude::*;
            let vbox = window
                .default_vbox()
                .expect("tao window should have default vbox");

            vbox.set_app_paintable(true);

            if let Some(visual) = vbox.screen().and_then(|s| s.rgba_visual()) {
                vbox.set_visual(Some(&visual));
            }

            use tao::platform::unix::WindowExtUnix;
            let gtk_window = window.gtk_window();
            gtk_window.set_default_size(config.width as i32, config.height as i32);

            if is_url {
                builder.with_url(&resolved_content).build_gtk(vbox)
            } else if is_file_path {
                let html_content = if let Some(cached) = HTML_CACHE.get(&resolved_content) {
                    cached.clone()
                } else {
                    match std::fs::read_to_string(&resolved_content) {
                        Ok(html) => {
                            HTML_CACHE.insert(resolved_content.clone(), html.clone());
                            html
                        }
                        Err(_) => {
                            eprintln!("[Error] 读取 HTML 文件失败：{}", resolved_content);
                            return;
                        }
                    }
                };

                let file_path = std::path::Path::new(&resolved_content);
                let absolute_path = if file_path.is_absolute() {
                    file_path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(file_path)
                };

                let base_dir = absolute_path.parent().unwrap_or(std::path::Path::new(""));
                let base_url = format!("file://{}/", base_dir.display());

                let html_with_base = if html_content.contains("<head>") {
                    html_content.replace("<head>", &format!("<head><base href=\"{}\">", base_url))
                } else if html_content.contains("<html>") {
                    html_content.replace(
                        "<html>",
                        &format!("<html><head><base href=\"{}\"></head>", base_url),
                    )
                } else {
                    format!(
                        "<html><head><base href=\"{}\"></head><body>{}</body></html>",
                        base_url, html_content
                    )
                };

                builder.with_html(&html_with_base).build_gtk(vbox)
            } else if is_dist {
                let dist_path = std::path::Path::new(&resolved_content);
                let index_html = dist_path.join("index.html");

                if !index_html.exists() {
                    eprintln!("[Error] dist 目录中不存在 index.html：{}", resolved_content);
                    return;
                }

                // 读取 HTML 并将所有路径改为 app:// 协议
                let html_content = match std::fs::read_to_string(&index_html) {
                    Ok(html) => {
                        eprintln!("[Window] 加载 dist 目录：{}", resolved_content);

                        // 将所有 href="/ 和 src="/ 改为 href="app:// 和 src="app://
                        let mut converted = html.replace("href=\"/", "href=\"app://");
                        converted = converted.replace("src=\"/", "src=\"app://");
                        converted = converted.replace("href='/", "href='app://");
                        converted = converted.replace("src='/", "src='app://");

                        eprintln!("[Window] 转换后使用 app:// 协议");
                        converted
                    }
                    Err(e) => {
                        eprintln!("[Error] 读取 dist/index.html 失败：{}", e);
                        return;
                    }
                };

                builder.with_html(&html_content).build_gtk(vbox)
            } else {
                builder
                    .with_html("<html><body>No content specified</body></html>")
                    .build_gtk(vbox)
            }
        };

        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        let result = {
            let t_build = std::time::Instant::now();
            let res = if is_url {
                eprintln!("[Performance][Rust] 使用 URL 模式加载内容");
                builder.with_url(&resolved_content).build(&window)
            } else if is_file_path {
                eprintln!("[Performance][Rust] 使用文件路径模式加载内容");
                let html_content = if let Some(cached) = HTML_CACHE.get(&resolved_content) {
                    cached.clone()
                } else {
                    match std::fs::read_to_string(&resolved_content) {
                        Ok(html) => {
                            HTML_CACHE.insert(resolved_content.clone(), html.clone());
                            html
                        }
                        Err(e) => {
                            eprintln!("[Error] 读取 HTML 文件失败：{}", e);
                            return;
                        }
                    }
                };

                let file_path = std::path::Path::new(&resolved_content);
                let absolute_path = if file_path.is_absolute() {
                    file_path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(file_path)
                };

                let base_dir = absolute_path.parent().unwrap_or(std::path::Path::new(""));

                #[cfg(target_os = "windows")]
                let base_url = format!(
                    "file:///{}/",
                    base_dir.display().to_string().replace("\\", "/")
                );

                #[cfg(not(target_os = "windows"))]
                let base_url = format!("file://{}/", base_dir.display());

                let html_with_base = if html_content.contains("<head>") {
                    html_content.replace("<head>", &format!("<head><base href=\"{}\">", base_url))
                } else if html_content.contains("<html>") {
                    html_content.replace(
                        "<html>",
                        &format!("<html><head><base href=\"{}\"></head>", base_url),
                    )
                } else {
                    format!(
                        "<html><head><base href=\"{}\"></head><body>{}</body></html>",
                        base_url, html_content
                    )
                };

                eprintln!("[Window] 使用 with_html 加载文件 | base_url={}", base_url);

                builder.with_html(&html_with_base).build(&window)
            } else if is_dist {
                eprintln!("[Performance][Rust] 使用 dist 目录模式加载内容");
                let dist_path = std::path::Path::new(&resolved_content);
                let index_html = dist_path.join("index.html");
                if !index_html.exists() {
                    eprintln!("[Error] dist 目录中不存在 index.html：{}", resolved_content);
                    return;
                }

                // 读取 HTML 并将所有路径改为 app:// 协议
                let html_content = match std::fs::read_to_string(&index_html) {
                    Ok(html) => {
                        eprintln!("[Window] 加载 dist 目录：{}", resolved_content);

                        #[cfg(target_os = "windows")]
                        let converted = {
                            html.replace("href=\"/", "href=\"http://app.")
                                .replace("src=\"/", "src=\"http://app.")
                                .replace("href='/", "href='http://app.")
                                .replace("src='/", "src='http://app.")
                        };

                        #[cfg(not(target_os = "windows"))]
                        let converted = {
                            html.replace("href=\"/", "href=\"app://")
                                .replace("src=\"/", "src=\"app://")
                                .replace("href='/", "href='app://")
                                .replace("src='/", "src='app://")
                        };

                        converted
                    }
                    Err(e) => {
                        eprintln!("[Error] 读取 dist/index.html 失败：{}", e);
                        return;
                    }
                };

                builder.with_html(&html_content).build(&window)
            } else {
                builder
                    .with_html("<html><body>No content specified</body></html>")
                    .build(&window)
            };
            eprintln!(
                "[Performance][Rust] WebView.build() 耗时: {:?}",
                t_build.elapsed()
            );
            res
        };

        match result {
            Ok(wv) => {
                eprintln!("[Window] WebView 创建成功！window_id={}", id_clone);
                eprintln!(
                    "[Performance][Rust] WebView 创建总耗时: {:?}",
                    t_webview_start.elapsed()
                );
                SendSyncWebView(std::sync::Arc::new(Mutex::new(Some(wv))))
            }
            Err(e) => {
                eprintln!("[Error] WebView 创建失败：{}", e);
                return;
            }
        }
    };

    #[cfg(target_os = "windows")]
    if !hwnd.is_null() {
        if config.enable_resizable && !config.show_title_bar {
            eprintln!(
                "[Window] 调用 make_window_frameless_but_resizable | enable_resizable={} | show_title_bar={}",
                config.enable_resizable, config.show_title_bar
            );
            crate::utils::make_window_frameless_but_resizable(windows::Win32::Foundation::HWND(
                hwnd,
            ));
        } else {
            eprintln!(
                "[Window] 跳过 make_window_frameless_but_resizable | enable_resizable={} | show_title_bar={}",
                config.enable_resizable, config.show_title_bar
            );
        }

        if config.show_title_bar {
            let corner_pref = match config.dwm_corner {
                1 => crate::configs::WindowCorners::DoNotRound,
                2 => crate::configs::WindowCorners::Round,
                3 => crate::configs::WindowCorners::RoundSmall,
                _ => crate::configs::WindowCorners::Default,
            };
            eprintln!("[Window] 设置 DWM 圆角: {:?}", corner_pref);
            match crate::utils::set_window_corner(
                windows::Win32::Foundation::HWND(hwnd),
                corner_pref,
            ) {
                Ok(_) => eprintln!("[Window] DWM 圆角设置成功"),
                Err(e) => eprintln!("[Window] DWM 圆角设置失败: {}", e),
            }
        } else {
            eprintln!("[Window] 禁用 DWM 圆角 (DoNotRound)");
            match crate::utils::set_window_corner(
                windows::Win32::Foundation::HWND(hwnd),
                crate::configs::WindowCorners::DoNotRound,
            ) {
                Ok(_) => eprintln!("[Window] DWM 圆角禁用成功"),
                Err(e) => eprintln!("[Window] DWM 圆角禁用失败: {}", e),
            }
        }
    }

    let t_storage = std::time::Instant::now();
    WINDOWS.insert(id_clone, window);
    WINDOW_PROXIES.insert(
        id_clone,
        WindowHandle {
            proxy: std::sync::Arc::new(Mutex::new(Some(proxy.clone()))),
        },
    );
    // 存储 proxy 用于跨线程发送消息
    EVENT_PROXIES.insert(id_clone, proxy.clone());
    // 存储 webview
    WEBVIEWS.insert(id_clone, webview);

    WINDOW_READY.insert(id_clone, true);
    eprintln!(
        "[Performance][Rust] 存储窗口数据耗时: {:?}",
        t_storage.elapsed()
    );
    eprintln!(
        "[Performance][Rust] create_window_in_event_loop 完成，总耗时: {:?}",
        t_total.elapsed()
    );
}

/// 获取所有窗口 ID 列表
pub fn get_all_window_ids() -> Vec<u64> {
    EVENT_PROXIES.iter().map(|entry| *entry.key()).collect()
}

struct WindowHandle {
    proxy: std::sync::Arc<Mutex<Option<tao::event_loop::EventLoopProxy<UserEvent>>>>,
}

impl WindowHandle {
    fn close(&self, window_id: u64) -> bool {
        if let Ok(proxy) = self.proxy.lock() {
            if let Some(ref p) = *proxy {
                return p.send_event(UserEvent::CloseWindow(window_id)).is_ok();
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub html_content: Option<String>,
    pub link_content: Option<String>,
    pub dist_content: Option<String>,
    pub icon_path: String,
    pub show_title_bar: bool,
    pub enable_resizable: bool,
    pub enable_devtools: bool,
    pub dwm_corner: u32,
}

fn create_window(config: WindowConfig) -> PyResult<u64> {
    let t_start = std::time::Instant::now();
    let window_id = generate_window_id();

    eprintln!(
        "[Performance][Rust] create_window 开始：{} | id={}",
        config.title, window_id
    );

    // 存储配置，让主事件循环可以创建窗口
    let t_pending = std::time::Instant::now();
    if let Ok(mut configs) = PENDING_WINDOWS.lock() {
        configs.insert(window_id, config);
        eprintln!("[Window] 窗口配置已添加到 PENDING_WINDOWS");
    }
    eprintln!(
        "[Performance][Rust] 添加到 PENDING_WINDOWS 耗时: {:?}",
        t_pending.elapsed()
    );

    WINDOW_READY.insert(window_id, false);

    // 唤醒事件循环，让它立即处理待创建的窗口
    // 这样可以确保运行时创建的窗口能够立即显示
    let t_wakeup = std::time::Instant::now();
    if let Ok(pending_configs) = PENDING_WINDOWS.lock() {
        if !pending_configs.is_empty() {
            drop(pending_configs);
            if let Ok(proxy_guard) = MAIN_EVENT_PROXY.lock() {
                if let Some(proxy) = proxy_guard.as_ref() {
                    eprintln!("[Window] 发送 WakeUp 事件到事件循环");
                    let _ = proxy.send_event(UserEvent::WakeUp);
                } else {
                    eprintln!("[Window] 警告：MAIN_EVENT_PROXY 为空");
                }
            } else {
                eprintln!("[Window] 警告：无法获取 MAIN_EVENT_PROXY 锁");
            }
        }
    }
    eprintln!(
        "[Performance][Rust] 唤醒事件循环耗时: {:?}",
        t_wakeup.elapsed()
    );
    eprintln!(
        "[Performance][Rust] create_window 完成，总耗时: {:?}",
        t_start.elapsed()
    );

    Ok(window_id)
}

/// 处理来自前端的 IPC 消息
fn handle_ipc_message(
    request: http::Request<String>,
    window_id: u64,
    proxy: &tao::event_loop::EventLoopProxy<UserEvent>,
) {
    let t_entry = std::time::Instant::now();
    let body = request.body();

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        let t_parse = t_entry.elapsed();
        if t_parse.as_micros() > 50 {
            eprintln!("[Timing][IPC] JSON 解析耗时: {:?}", t_parse);
        }

        if let Some(obj) = value.as_object() {
            let handle_id = obj
                .get("handle_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let handle_type = obj
                .get("handle_type")
                .and_then(|v| v.as_str())
                .unwrap_or("invoke")
                .to_string();
            let request_id = obj
                .get("request_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let payload = obj
                .get("payload")
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            match handle_type.as_str() {
                "invoke" => {
                    // 特殊处理：Linux 窗口拖动（通过内部函数，不导出到Python）
                    #[cfg(any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd"
                    ))]
                    if handle_id == "__rust_start_drag_window" {
                        let button =
                            payload.get("button").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                        let _ = internal_start_drag(window_id, button);
                        let response = serde_json::json!({
                            "window_id": window_id,
                            "handle_id": handle_id,
                            "handle_type": "invoke",
                            "request_id": request_id,
                            "payload": {"code": 200, "mssg": "ok", "data": null}
                        });
                        let js_code = format!(
                            "window.__pywebron_dispatch({})",
                            serde_json::to_string(&response).unwrap_or_default()
                        );
                        let _ = proxy.send_event(UserEvent::EvaluateScript {
                            window_id,
                            script: js_code,
                        });
                        return;
                    }

                    // 特殊处理：Windows 无边框窗口调整大小
                    #[cfg(target_os = "windows")]
                    if handle_id == "__rust_start_resize" {
                        use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
                        use windows::Win32::UI::WindowsAndMessaging::{
                            SendMessageW, WM_NCLBUTTONDOWN,
                        };

                        let ht = payload
                            .get("hit_test")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize;
                        let win_id = payload
                            .get("window_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        if let Some(window) = WINDOWS.get(&win_id) {
                            use tao::platform::windows::WindowExtWindows;
                            let hwnd = windows::Win32::Foundation::HWND(
                                window.hwnd() as *mut std::ffi::c_void
                            );
                            unsafe {
                                let _ = ReleaseCapture();
                                let _ = SendMessageW(
                                    hwnd,
                                    WM_NCLBUTTONDOWN,
                                    Some(windows::Win32::Foundation::WPARAM(ht)),
                                    Some(windows::Win32::Foundation::LPARAM(0)),
                                );
                            }
                        }

                        // 发送成功响应
                        let response = serde_json::json!({
                            "window_id": window_id,
                            "handle_id": handle_id,
                            "handle_type": "invoke",
                            "request_id": request_id,
                            "payload": {"code": 200, "mssg": "ok", "data": null}
                        });
                        let js_code = format!(
                            "window.__pywebron_dispatch({})",
                            serde_json::to_string(&response).unwrap_or_default()
                        );
                        let _ = proxy.send_event(UserEvent::EvaluateScript {
                            window_id,
                            script: js_code,
                        });
                        return;
                    }

                    // 特殊处理：Linux GTK 无边框窗口调整大小
                    #[cfg(any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd"
                    ))]
                    if handle_id == "__rust_start_resize" {
                        use gtk::gdk::WindowEdge;
                        use gtk::prelude::{SeatExt, WidgetExt};

                        let ht = payload
                            .get("hit_test")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i32;
                        let win_id = payload
                            .get("window_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        if let Some(window) = WINDOWS.get(&win_id) {
                            use tao::platform::unix::WindowExtUnix;
                            let gtk_window = window.gtk_window();
                            if let Some(gdk_window) = gtk_window.window() {
                                let edge = match ht {
                                    10 => WindowEdge::West,
                                    11 => WindowEdge::East,
                                    12 => WindowEdge::North,
                                    13 => WindowEdge::NorthWest,
                                    14 => WindowEdge::NorthEast,
                                    15 => WindowEdge::South,
                                    16 => WindowEdge::SouthWest,
                                    17 => WindowEdge::SouthEast,
                                    _ => WindowEdge::West,
                                };
                                if let Some(device) = gdk_window
                                    .display()
                                    .default_seat()
                                    .and_then(|s| s.pointer())
                                {
                                    gdk_window.begin_resize_drag_for_device(
                                        edge, &device, 0, // button
                                        0, // root_x
                                        0, // root_y
                                        0, // timestamp (0 = current time)
                                    );
                                }
                            }
                        }

                        // 发送成功响应
                        let response = serde_json::json!({
                            "window_id": window_id,
                            "handle_id": handle_id,
                            "handle_type": "invoke",
                            "request_id": request_id,
                            "payload": {"code": 200, "mssg": "ok", "data": null}
                        });
                        let js_code = format!(
                            "window.__pywebron_dispatch({})",
                            serde_json::to_string(&response).unwrap_or_default()
                        );
                        let _ = proxy.send_event(UserEvent::EvaluateScript {
                            window_id,
                            script: js_code,
                        });
                        return;
                    }

                    // 提交到 invoke 线程池，结果自动通过 proxy 发回前端
                    crate::app::invoke::submit_invoke_ipc(
                        handle_id,
                        window_id,
                        request_id,
                        payload,
                        proxy.clone(),
                    );
                }
                "stream" => {
                    use crate::app::stream::{
                        is_handler_active, is_stream_active, push_stream_data,
                        register_stream_window,
                    };

                    // Stream 处理：区分"启动"和"数据"消息
                    if is_stream_active(&handle_id, window_id) {
                        // 该窗口已订阅：直接投递数据到队列
                        push_stream_data(&handle_id, window_id, payload);
                    } else if is_handler_active(&handle_id) {
                        // handler 已活跃：注册新窗口订阅并推送数据
                        register_stream_window(&handle_id, window_id);
                        push_stream_data(&handle_id, window_id, payload);
                    } else {
                        // handler 未启动：提交到 stream 线程池启动
                        crate::app::invoke::submit_stream_ipc(
                            handle_id, window_id, request_id, payload,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

#[pyfunction(name = "rust_init")]
#[pyo3(signature = (prewarm_webview=false))]
pub fn init(prewarm_webview: bool) -> PyResult<()> {
    let t_start = std::time::Instant::now();
    eprintln!("[Performance][Rust] rust_init 开始");

    #[cfg(target_os = "windows")]
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "00000000");

    #[cfg(target_os = "windows")]
    if prewarm_webview {
        let t_prewarm = std::time::Instant::now();
        prewarm_webview2();
        eprintln!(
            "[Performance][Rust] prewarm_webview2 耗时: {:?}",
            t_prewarm.elapsed()
        );
    }

    eprintln!(
        "[Performance][Rust] rust_init 完成，总耗时: {:?}",
        t_start.elapsed()
    );
    Ok(())
}

#[cfg(target_os = "windows")]
fn prewarm_webview2() {
    std::thread::spawn(move || {
        let event = EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build();

        let window = match WindowBuilder::new()
            .with_visible(false)
            .with_transparent(true) // 关键：预热窗口也要透明
            .with_decorations(false) // 关键：无装饰
            .with_inner_size(LogicalSize::new(1, 1))
            .build(&event)
        {
            Ok(w) => w,
            Err(_) => return,
        };

        // 关键：创建透明的 WebView，确保 WebView2 进程使用透明设置
        let _webview = WebViewBuilder::new()
            .with_transparent(true)
            .with_html("<html><body style='background:transparent'></body></html>")
            .build(&window)
            .ok();

        // 保持事件循环运行以维持 WebView2 Runtime 进程
        event.run(|_, _, flow| {
            *flow = ControlFlow::Wait;
        });
    });
}

#[pyfunction(name = "rust_register_window")]
#[pyo3(signature = (title, width, height, html_content, link_content, dist_content, icon_path, show_title_bar, enable_resizable, enable_devtools, dwm_corner=0))]
pub fn register_window(
    title: String,
    width: u32,
    height: u32,
    html_content: Option<String>,
    link_content: Option<String>,
    dist_content: Option<String>,
    icon_path: String,
    show_title_bar: bool,
    enable_resizable: bool,
    enable_devtools: bool,
    dwm_corner: u32,
) -> PyResult<bool> {
    let t_start = std::time::Instant::now();
    eprintln!("[Performance][Rust] rust_register_window 开始: {}", title);

    let config = WindowConfig {
        title: title.clone(),
        width,
        height,
        html_content,
        link_content,
        dist_content,
        icon_path,
        show_title_bar,
        enable_resizable,
        enable_devtools,
        dwm_corner,
    };

    // 直接创建窗口
    let t_create = std::time::Instant::now();
    let window_id = create_window(config.clone())?;
    eprintln!(
        "[Performance][Rust] create_window 耗时: {:?}",
        t_create.elapsed()
    );

    // 存储配置供 get_windows() 查询
    if let Ok(mut configs) = WINDOW_CONFIGS.write() {
        configs.insert(window_id, config);
    }

    eprintln!(
        "[Performance][Rust] rust_register_window 完成: {}, 总耗时: {:?}",
        title,
        t_start.elapsed()
    );
    Ok(true)
}

// 存储窗口配置，用于 get_windows 返回
use std::sync::RwLock;

static WINDOW_CONFIGS: Lazy<RwLock<HashMap<u64, WindowConfig>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 运行主循环（在 Python 主线程运行 TAO 事件循环）
#[pyfunction(name = "rust_run")]
pub fn run(_py: Python<'_>) -> PyResult<()> {
    let t_start = std::time::Instant::now();
    eprintln!("[Performance][Rust] rust_run 开始");

    // 创建全局事件循环（必须在主线程）
    let t_event_loop = std::time::Instant::now();
    #[cfg(target_os = "windows")]
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .with_any_thread(true)
        .build();

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .with_any_thread(true)
        .build();

    #[cfg(target_os = "macos")]
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    eprintln!(
        "[Performance][Rust] 事件循环创建耗时: {:?}",
        t_event_loop.elapsed()
    );

    let proxy = event_loop.create_proxy();

    // 存储全局 proxy，用于唤醒事件循环
    if let Ok(mut main_proxy) = MAIN_EVENT_PROXY.lock() {
        *main_proxy = Some(proxy.clone());
    }

    eprintln!("[Window] 启动事件循环（主线程）");
    eprintln!(
        "[Performance][Rust] rust_run 初始化完成，耗时: {:?}",
        t_start.elapsed()
    );

    // 关键：释放 GIL，允许其他线程（invoke/stream 线程池）访问 Python
    // event_loop.run() 是阻塞调用，如果不释放 GIL，其他线程的 Python::attach() 会死锁
    // 使用 PyEval_SaveThread/PyEval_RestoreThread 手动管理 GIL
    let _thread_state = unsafe { PyEval_SaveThread() };

    event_loop.run(move |event, event_loop, flow| {
        *flow = ControlFlow::Wait;

        // 处理窗口关闭事件
        match &event {
            Event::UserEvent(UserEvent::WakeUp) => {
                let t_wakeup = std::time::Instant::now();
                eprintln!("[Window][Event] 收到 WakeUp 事件，准备处理待创建的窗口");
                // 唤醒事件，立即处理待创建的窗口
                if let Ok(mut pending) = PENDING_WINDOWS.lock() {
                    let count = pending.len();
                    eprintln!(
                        "[Window][Event] 获取到 PENDING_WINDOWS，共 {} 个窗口",
                        count
                    );
                    if count > 0 {
                        for (window_id, config) in pending.drain() {
                            eprintln!(
                                "[Window][Event] 开始创建窗口：{} | id={}",
                                config.title, window_id
                            );
                            let t_create = std::time::Instant::now();
                            create_window_in_event_loop(&config, window_id, &proxy, event_loop);
                            eprintln!(
                                "[Performance][Rust] 窗口创建完成：{} | id={} | 耗时: {:?}",
                                config.title,
                                window_id,
                                t_create.elapsed()
                            );
                        }
                    }
                }
                eprintln!(
                    "[Performance][Rust] WakeUp 事件处理完成，耗时: {:?}",
                    t_wakeup.elapsed()
                );
            }
            Event::NewEvents(_) => {
                // NewEvents 时也检查待创建窗口（处理事件循环自然唤醒的情况）
                if let Ok(mut pending) = PENDING_WINDOWS.lock() {
                    let count = pending.len();
                    if count > 0 {
                        eprintln!("[Window][Event] NewEvents 发现 {} 个待创建窗口", count);
                        for (window_id, config) in pending.drain() {
                            create_window_in_event_loop(&config, window_id, &proxy, event_loop);
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::CloseWindow(window_id)) => {
                cleanup_window(*window_id);
                if WINDOWS.is_empty() {
                    *flow = ControlFlow::Exit;
                }
            }
            Event::WindowEvent {
                window_id,
                event: CloseRequested,
                ..
            } => {
                // 通过 tao WindowId 查找内部 u64 ID
                let internal_id = WINDOWS.iter().find_map(|entry| {
                    if entry.value().id() == *window_id {
                        Some(*entry.key())
                    } else {
                        None
                    }
                });
                if let Some(id) = internal_id {
                    cleanup_window(id);
                }
                if WINDOWS.is_empty() {
                    *flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(UserEvent::EvaluateScript { window_id, script }) => {
                // 处理跨线程发送的消息
                if let Some(webview) = WEBVIEWS.get(window_id) {
                    if let Ok(guard) = webview.0.lock() {
                        if let Some(wv) = guard.as_ref() {
                            let _ = wv.evaluate_script(&script);
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            Event::UserEvent(UserEvent::SetCorner { .. }) => {
                // 已禁用圆角功能
            }
            _ => {}
        }
    });

    // event_loop.run() 在 ControlFlow::Exit 时返回，恢复 GIL
    #[allow(unreachable_code)]
    {
        unsafe { PyEval_RestoreThread(_thread_state) };
    }

    Ok(())
}

#[pyfunction(name = "rust_minimize_window")]
pub fn minimize_window(id: u64) -> PyResult<bool> {
    let window = WINDOWS
        .get(&id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("窗口 {} 不存在", id)))?;
    window.set_minimized(true);
    Ok(true)
}

/// Linux专用：开始窗口拖动（通过GTK原生API）- 内部函数，不导出到Python
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn internal_start_drag(id: u64, button: u32) -> PyResult<bool> {
    use gtk::prelude::GtkWindowExt;

    let window = WINDOWS
        .get(&id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("窗口 {} 不存在", id)))?;

    let gtk_window = window.gtk_window();
    let timestamp = gtk::glib::monotonic_time() as u32;
    gtk_window.begin_move_drag(button as i32, 0, 0, timestamp);
    Ok(true)
}

#[pyfunction(name = "rust_dragdrop_window")]
pub fn dragdrop_window(id: u64, selector: &str) -> PyResult<bool> {
    let proxy = EVENT_PROXIES
        .get(&id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("窗口 {} 不存在", id)))?;

    #[cfg(target_os = "windows")]
    {
        let js_code = format!(
            r#"(function() {{
                const el = document.querySelector('{}');
                if (el) {{
                    el.style.webkitAppRegion = 'drag';
                    el.querySelectorAll('button, input, [onclick], .win-btn').forEach(child => {{
                        child.style.webkitAppRegion = 'no-drag';
                    }});
                }}
            }})()"#,
            selector
        );
        let _ = proxy.send_event(UserEvent::EvaluateScript {
            window_id: id,
            script: js_code,
        });
    }

    #[cfg(target_os = "macos")]
    {
        let js_code = format!(
            r#"(function() {{
                const el = document.querySelector('{}');
                if (el) {{
                    el.style.webkitAppRegion = 'drag';
                    el.querySelectorAll('button, input, [onclick], .win-btn').forEach(child => {{
                        child.style.webkitAppRegion = 'no-drag';
                    }});
                }}
            }})()"#,
            selector
        );
        let _ = proxy.send_event(UserEvent::EvaluateScript {
            window_id: id,
            script: js_code,
        });
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        let js_code = format!(
            r#"(function() {{
                const el = document.querySelector('{}');
                if (el) {{
                    el.addEventListener('mousedown', (e) => {{
                        if (e.target.closest('button, input, [onclick], .win-btn')) return;
                        if (window.pywebron && window.pywebron.invoke) {{
                            window.pywebron.invoke('__rust_start_drag_window', {{
                                window_id: window.pywebron.window_id,
                                button: 1
                            }});
                        }}
                    }});
                    el.addEventListener('dblclick', (e) => {{
                        if (e.target.closest('button, input, [onclick], .win-btn')) return;
                        if (window.pywebron && window.pywebron.invoke) {{
                            window.pywebron.invoke('window_controls_invoke', {{ control_type: 'toggle' }});
                        }}
                    }});
                }}
            }})()"#,
            selector
        );
        let _ = proxy.send_event(UserEvent::EvaluateScript {
            window_id: id,
            script: js_code,
        });
    }

    Ok(true)
}

#[pyfunction(name = "rust_maximize_window")]
pub fn maximize_window(id: u64) -> PyResult<bool> {
    let window = WINDOWS
        .get(&id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("窗口 {} 不存在", id)))?;
    window.set_maximized(true);
    Ok(true)
}

#[pyfunction(name = "rust_reappear_window")]
pub fn reappear_window(id: u64) -> PyResult<bool> {
    let window = WINDOWS
        .get(&id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("窗口 {} 不存在", id)))?;
    window.set_maximized(false);
    Ok(true)
}

#[pyfunction(name = "rust_shutdown_window")]
pub fn shutdown_window(id: u64) -> PyResult<bool> {
    if let Some(handle) = WINDOW_PROXIES.get(&id) {
        if handle.close(id) {
            return Ok(true);
        }
    }
    WINDOWS.remove(&id);
    Ok(true)
}

#[pyfunction(name = "rust_get_windows")]
pub fn get_windows(py: Python<'_>) -> PyResult<Bound<'_, pyo3::types::PyDict>> {
    let result_dict = pyo3::types::PyDict::new(py);

    if let Ok(configs) = WINDOW_CONFIGS.read() {
        for (window_id, config) in configs.iter() {
            let window_dict = pyo3::types::PyDict::new(py);
            window_dict.set_item("window_title", &config.title)?;
            window_dict.set_item("window_width", config.width)?;
            window_dict.set_item("window_height", config.height)?;
            window_dict.set_item("window_html_content", &config.html_content)?;
            window_dict.set_item("window_link_content", &config.link_content)?;
            window_dict.set_item("window_dist_content", &config.dist_content)?;
            window_dict.set_item("window_icon_path", &config.icon_path)?;
            window_dict.set_item("window_show_title_bar", config.show_title_bar)?;
            window_dict.set_item("window_enable_resizable", config.enable_resizable)?;
            window_dict.set_item("window_enable_devtools", config.enable_devtools)?;

            result_dict.set_item(*window_id, window_dict)?;
        }
    }

    Ok(result_dict)
}

/// 保存文件对话框（异步版本，不阻塞 Python 事件循环）
/// 复制源文件到用户选择的新位置
#[pyfunction(name = "rust_save_file_dialog")]
#[pyo3(signature = (source_file_path, new_file_name=None, is_del_source_file=false))]
pub fn save_file_dialog(
    py: Python,
    source_file_path: String,
    new_file_name: Option<String>,
    is_del_source_file: bool,
) -> PyResult<Bound<PyAny>> {
    use pyo3_async_runtimes::tokio::future_into_py;
    use rfd::FileDialog;
    use std::fs;
    use std::path::Path;

    // 使用 future_into_py 将异步 Rust 代码转换为 Python 可 await 的对象
    future_into_py(py, async move {
        // 在 tokio spawn_blocking 中运行，避免阻塞 async runtime
        let result = tokio::task::spawn_blocking(move || -> PyResult<Option<String>> {
            // 获取源文件信息
            let source_path = Path::new(&source_file_path);

            // 检查源文件是否存在
            if !source_path.exists() {
                return Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                    "源文件不存在: {}",
                    source_file_path
                )));
            }

            // 获取源文件的文件名和扩展名
            let source_file_name = source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let source_extension = source_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            // 确定新文件名：用户传递的优先，否则使用源文件名
            let final_file_name = new_file_name.unwrap_or_else(|| source_file_name.to_string());

            // 构建文件过滤器（基于源文件类型）
            let file_filter_name = if source_extension.is_empty() {
                "所有文件".to_string()
            } else {
                format!("{} 文件", source_extension.to_uppercase())
            };

            // 打开保存对话框
            let mut dialog = FileDialog::new();
            dialog = dialog.set_file_name(&final_file_name);

            // 添加文件类型过滤器
            if !source_extension.is_empty() {
                dialog = dialog.add_filter(&file_filter_name, &[source_extension]);
            }
            dialog = dialog.add_filter("所有文件", &["*"]);

            // 显示对话框并获取用户选择的路径
            let target_path = match dialog.save_file() {
                Some(path) => path,
                None => return Ok(None), // 用户取消
            };

            // 复制源文件到目标路径
            match fs::copy(&source_path, &target_path) {
                Ok(bytes_copied) => {
                    // 如果需要删除源文件
                    if is_del_source_file {
                        if let Err(e) = fs::remove_file(&source_path) {
                            eprintln!("[Rust] 删除源文件失败: {}", e);
                            // 复制成功但删除失败，返回目标路径但打印警告
                        }
                    }

                    println!(
                        "[Rust] 文件复制成功: {} -> {} ({} bytes)",
                        source_file_path,
                        target_path.display(),
                        bytes_copied
                    );

                    Ok(Some(target_path.to_string_lossy().to_string()))
                }
                Err(e) => Err(pyo3::exceptions::PyIOError::new_err(format!(
                    "文件复制失败: {}",
                    e
                ))),
            }
        })
        .await;

        // 处理结果：将 Rust 错误直接传递给 Python
        match result {
            Ok(Ok(path)) => Ok(path),
            Ok(Err(py_err)) => Err(py_err),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "对话框任务失败: {}",
                e
            ))),
        }
    })
}

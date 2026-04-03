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

fn cleanup_window(window_id: u64) {
    WINDOWS.remove(&window_id);
    WINDOW_PROXIES.remove(&window_id);
    WINDOW_READY.remove(&window_id);
    EVENT_PROXIES.remove(&window_id);
    WEBVIEWS.remove(&window_id);
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
    let id_clone = window_id;

    #[cfg(target_os = "windows")]
    let window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_window_icon(generate_win_icon(config.icon_path.clone()))
        .with_decorations(config.decorations)
        .with_resizable(config.resizable)
        .with_min_inner_size(LogicalSize::new(400u32, 300u32))
        .with_transparent(true)
        .with_undecorated_shadow(false);

    eprintln!("[Window] 创建窗口：{} | decorations={} | size={}x{}", 
              config.title, config.decorations, config.width, config.height);

    #[cfg(not(target_os = "windows"))]
    let window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(PhysicalSize::new(config.width, config.height))
        .with_window_icon(generate_win_icon(config.icon_path.clone()))
        .with_decorations(config.decorations)
        .with_resizable(config.resizable)
        .with_min_inner_size(PhysicalSize::new(400u32, 300u32))
        .with_transparent(true);

    let window = match window_builder.build(event_loop) {
        Ok(w) => w,
        Err(_) => {
            eprintln!("[Error] 窗口创建失败：{}", config.title);
            return;
        }
    };

    #[cfg(target_os = "windows")]
    let hwnd = {
        use tao::platform::windows::WindowExtWindows;
        let hwnd = window.hwnd() as *mut std::ffi::c_void;

        // Windows 透明背景设置
        use windows::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH};
        use windows::Win32::UI::WindowsAndMessaging::{SetClassLongPtrW, GCLP_HBRBACKGROUND};

        unsafe {
            let win_hwnd = windows::Win32::Foundation::HWND(hwnd);

            // 设置窗口背景为黑色（DWM 会将黑色视为透明）
            let hbrush = GetStockObject(BLACK_BRUSH);
            let _ = SetClassLongPtrW(win_hwnd, GCLP_HBRBACKGROUND, hbrush.0 as isize);

            // 如果窗口可调整大小，确保有 WS_THICKFRAME 样式
            if config.resizable {
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
    let webview = {
        let _webview_lock = WEBVIEW_CREATE_LOCK.lock().unwrap();

        let is_url =
            config.content.starts_with("http://") || config.content.starts_with("https://");
        let is_file_path = !is_url
            && (config.content.ends_with(".html")
                || config.content.ends_with(".htm")
                || std::path::Path::new(&config.content).exists());

        let proxy_for_handler = proxy.clone();
        let window_id_for_ipc = id_clone;
        let builder = WebViewBuilder::new()
            .with_devtools(config.devtools)
            .with_transparent(true)
            .with_background_color((0, 0, 0, 0)) // RGBA: 透明背景
            .with_initialization_script(&format!(
                "window.pywebron={{window_id:{},hasSystemTitleBar:{}}};{}",
                id_clone,
                config.decorations,
                load_js_api()
            ))
            .with_ipc_handler(move |request| {
                // 处理来自前端的 IPC 消息
                handle_ipc_message(request, window_id_for_ipc, &proxy_for_handler);
            });

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        let result = {
            // Linux: 使用 build_gtk() 同时支持 X11 和 Wayland
            // 使用 default_vbox 作为容器，避免与 tao 默认布局冲突
            let vbox = window
                .default_vbox()
                .expect("tao window should have default vbox");

            // 设置 GTK 容器背景透明（关键：解决 Linux 下 WebView 背景白色问题）
            // 参考：https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_transparent
            use gtk::prelude::*;
            vbox.set_app_paintable(true);

            // 获取带 alpha 通道的 visual
            if let Some(visual) = vbox.screen().and_then(|s| s.rgba_visual()) {
                vbox.set_visual(Some(&visual));
            }
            
            // 强制设置 GTK 窗口的缩放因子为 1.0，避免 DPI 缩放问题
            use tao::platform::unix::WindowExtUnix;
            let gtk_window = window.gtk_window();
            // 显式设置窗口大小，确保使用物理像素
            gtk_window.set_default_size(config.width as i32, config.height as i32);

            if is_url {
                builder.with_url(&config.content).build_gtk(vbox)
            } else if is_file_path {
                let html_content = if let Some(cached) = HTML_CACHE.get(&config.content) {
                    cached.clone()
                } else {
                    match std::fs::read_to_string(&config.content) {
                        Ok(html) => {
                            HTML_CACHE.insert(config.content.clone(), html.clone());
                            html
                        }
                        Err(_) => {
                            eprintln!("[Error] 读取 HTML 文件失败：{}", config.content);
                            return;
                        }
                    }
                };
                builder.with_html(&html_content).build_gtk(vbox)
            } else {
                builder.with_html(&config.content).build_gtk(vbox)
            }
        };

        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        let result = if is_url {
            builder.with_url(&config.content).build(&window)
        } else if is_file_path {
            // 使用缓存避免重复读取文件
            let html_content = if let Some(cached) = HTML_CACHE.get(&config.content) {
                cached.clone()
            } else {
                match std::fs::read_to_string(&config.content) {
                    Ok(html) => {
                        HTML_CACHE.insert(config.content.clone(), html.clone());
                        html
                    }
                    Err(_) => {
                        eprintln!("[Error] 读取 HTML 文件失败：{}", config.content);
                        return;
                    }
                }
            };
            
            // 关键：使用 data URL 而不是直接 with_html，这样可以提供 base URL
            let file_path = std::path::Path::new(&config.content);
            let absolute_path = if file_path.is_absolute() {
                file_path.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(file_path)
            };
            
            // 获取文件所在目录作为 base URL
            let base_dir = absolute_path.parent().unwrap_or(std::path::Path::new(""));
            
            #[cfg(target_os = "windows")]
            let base_url = format!("file:///{}/", base_dir.display().to_string().replace("\\", "/"));
            
            #[cfg(not(target_os = "windows"))]
            let base_url = format!("file://{}/", base_dir.display());
            
            eprintln!("[Window] HTML base URL: {}", base_url);
            
            // 在 HTML 中注入 base 标签
            let html_with_base = if html_content.contains("<head>") {
                html_content.replace("<head>", &format!("<head><base href=\"{}\">", base_url))
            } else if html_content.contains("<html>") {
                html_content.replace("<html>", &format!("<html><head><base href=\"{}\"></head>", base_url))
            } else {
                format!("<html><head><base href=\"{}\"></head><body>{}</body></html>", base_url, html_content)
            };
            
            builder.with_html(&html_with_base).build(&window)
        } else {
            builder.with_html(&config.content).build(&window)
        };

        match result {
            Ok(wv) => SendSyncWebView(std::sync::Arc::new(Mutex::new(Some(wv)))),
            Err(e) => {
                eprintln!("[Error] WebView 创建失败：{}", e);
                return;
            }
        }
    };

    #[cfg(target_os = "windows")]
    if !hwnd.is_null() {
        if config.resizable && !config.decorations {
            eprintln!("[Window] 调用 make_window_frameless_but_resizable | resizable={} | decorations={}", 
                      config.resizable, config.decorations);
            crate::utils::make_window_frameless_but_resizable(windows::Win32::Foundation::HWND(hwnd));
        } else {
            eprintln!("[Window] 跳过 make_window_frameless_but_resizable | resizable={} | decorations={}", 
                      config.resizable, config.decorations);
        }
    }

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
    pub content: String,
    pub icon_path: String,
    pub decorations: bool,
    pub resizable: bool,
    pub devtools: bool,
}

fn create_window(config: WindowConfig) -> PyResult<u64> {
    let window_id = generate_window_id();

    eprintln!(
        "[Window] 准备创建新窗口：{} | id={}",
        config.title, window_id
    );

    // 存储配置，让主事件循环可以创建窗口
    if let Ok(mut configs) = PENDING_WINDOWS.lock() {
        configs.insert(window_id, config);
        eprintln!("[Window] 窗口配置已添加到 PENDING_WINDOWS");
    }

    WINDOW_READY.insert(window_id, false);

    // 唤醒事件循环，让它立即处理待创建的窗口
    // 这样可以确保运行时创建的窗口能够立即显示
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
                    // 特殊处理：Linux 窗口拖动（直接在 Rust 端处理，不经过 Python）
                    if handle_id == "__rust_start_drag_window" {
                        let button =
                            payload.get("button").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                        let _ = start_drag_window(window_id, button, 0, 0);
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
                                        
                    // 特殊处理：Windows 无边框窗口调整大小
                    #[cfg(target_os = "windows")]
                    if handle_id == "__rust_start_resize" {
                        use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_NCLBUTTONDOWN};
                        use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;

                        let ht = payload.get("hit_test").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let win_id = payload.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);

                        if let Some(window) = WINDOWS.get(&win_id) {
                            use tao::platform::windows::WindowExtWindows;
                            let hwnd = windows::Win32::Foundation::HWND(window.hwnd() as *mut std::ffi::c_void);
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
                        use gtk::prelude::{WidgetExt, SeatExt};
                        
                        let ht = payload.get("hit_test").and_then(|v| v.as_u64()).unwrap_or(0) as i32;
                        let win_id = payload.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
                        
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
                                if let Some(device) = gdk_window.display().default_seat().and_then(|s| s.pointer()) {
                                    gdk_window.begin_resize_drag_for_device(
                                        edge,
                                        &device,
                                        0, // button
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
pub fn init() -> PyResult<()> {
    // 关键：设置 WebView2 透明背景环境变量（必须在创建任何 WebView 之前）
    // 格式：AARRGGBB，00 表示完全透明
    #[cfg(target_os = "windows")]
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "00000000");

    // 预热 WebView2 Runtime（后台线程，不阻塞）
    // 暂时禁用预热，避免出现额外的窗口
    // #[cfg(target_os = "windows")]
    // prewarm_webview2();

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
#[pyo3(signature = (title, width, height, content, icon_path, decorations, resizable, devtools))]
pub fn register_window(
    title: String,
    width: u32,
    height: u32,
    content: String,
    icon_path: String,
    decorations: bool,
    resizable: bool,
    devtools: bool,
) -> PyResult<bool> {
    let config = WindowConfig {
        title,
        width,
        height,
        content,
        icon_path,
        decorations,
        resizable,
        devtools,
    };

    // 直接创建窗口
    let window_id = create_window(config.clone())?;

    // 存储配置供 get_windows() 查询
    if let Ok(mut configs) = WINDOW_CONFIGS.write() {
        configs.insert(window_id, config);
    }

    Ok(true)
}

// 存储窗口配置，用于 get_windows 返回
use std::sync::RwLock;

static WINDOW_CONFIGS: Lazy<RwLock<HashMap<u64, WindowConfig>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 运行主循环（在 Python 主线程运行 TAO 事件循环）
#[pyfunction(name = "rust_run")]
pub fn run(_py: Python<'_>) -> PyResult<()> {
    // 创建全局事件循环（必须在主线程）
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

    let proxy = event_loop.create_proxy();

    // 存储全局 proxy，用于唤醒事件循环
    if let Ok(mut main_proxy) = MAIN_EVENT_PROXY.lock() {
        *main_proxy = Some(proxy.clone());
    }

    eprintln!("[Window] 启动事件循环（主线程）");

    // 关键：释放 GIL，允许其他线程（invoke/stream 线程池）访问 Python
    // event_loop.run() 是阻塞调用，如果不释放 GIL，其他线程的 Python::attach() 会死锁
    // 使用 PyEval_SaveThread/PyEval_RestoreThread 手动管理 GIL
    let _thread_state = unsafe { PyEval_SaveThread() };

    event_loop.run(move |event, event_loop, flow| {
        *flow = ControlFlow::Wait;

        // 处理窗口关闭事件
        match &event {
            Event::UserEvent(UserEvent::WakeUp) => {
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
                            create_window_in_event_loop(&config, window_id, &proxy, event_loop);
                            eprintln!(
                                "[Window][Event] 窗口创建完成：{} | id={}",
                                config.title, window_id
                            );
                        }
                    }
                }
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

/// Linux专用：开始窗口拖动（通过GTK原生API）
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
#[pyfunction(name = "rust_start_drag_window")]
pub fn start_drag_window(id: u64, button: u32, _x: i32, _y: i32) -> PyResult<bool> {
    use gtk::prelude::GtkWindowExt;

    let window = WINDOWS
        .get(&id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("窗口 {} 不存在", id)))?;

    // 获取GTK窗口并开始拖动
    let gtk_window = window.gtk_window();
    // 获取当前时间戳并开始拖动
    let timestamp = gtk::glib::monotonic_time() as u32;
    // button: 1=左键, 2=中键, 3=右键
    gtk_window.begin_move_drag(button as i32, 0, 0, timestamp);
    Ok(true)
}

/// 非Linux平台：空实现
#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
#[pyfunction(name = "rust_start_drag_window")]
pub fn start_drag_window(_id: u64, _button: u32, _x: i32, _y: i32) -> PyResult<bool> {
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
            window_dict.set_item("window_content_path", &config.content)?;
            window_dict.set_item("window_icon_path", &config.icon_path)?;
            window_dict.set_item("window_is_decorations", config.decorations)?;
            window_dict.set_item("window_is_resizable", config.resizable)?;
            window_dict.set_item("window_is_devtools", config.devtools)?;

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

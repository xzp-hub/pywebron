use crate::app::load_js_api;
use crate::configs::{debug_log, UserEvent};
use crate::utils::generate_win_icon;
use dashmap::DashMap;
use pyo3::ffi::{PyEval_RestoreThread, PyEval_SaveThread};
use pyo3::prelude::*;
use pyo3::{Bound, PyResult};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
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

static WEBVIEW_CREATE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static WINDOWS: LazyLock<DashMap<u64, tao::window::Window>> = LazyLock::new(DashMap::new);
static WINDOW_PROXIES: LazyLock<DashMap<u64, WindowHandle>> = LazyLock::new(DashMap::new);
static WINDOW_READY: LazyLock<DashMap<u64, bool>> = LazyLock::new(DashMap::new);
static EVENT_PROXIES: LazyLock<DashMap<u64, tao::event_loop::EventLoopProxy<UserEvent>>> =
    LazyLock::new(DashMap::new);
static MAIN_EVENT_PROXY: LazyLock<Mutex<Option<tao::event_loop::EventLoopProxy<UserEvent>>>> =
    LazyLock::new(|| Mutex::new(None));
static PENDING_WINDOWS: LazyLock<Mutex<HashMap<u64, WindowConfig>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static MAIN_WINDOW_ID: LazyLock<Mutex<Option<u64>>> = LazyLock::new(|| Mutex::new(None));
// SAFETY: WebView contains non-Send raw pointers, but all access is guarded by the inner Mutex.
// Only event-loop thread touches the WebView through the Mutex; other threads only hold the Arc.
struct WebViewWrapper(std::sync::Arc<Mutex<Option<wry::WebView>>>);
unsafe impl Send for WebViewWrapper {}
unsafe impl Sync for WebViewWrapper {}

static WEBVIEWS: LazyLock<DashMap<u64, WebViewWrapper>> = LazyLock::new(DashMap::new);
static WINDOW_CACHE_KEYS: LazyLock<DashMap<u64, HashSet<String>>> = LazyLock::new(DashMap::new);

/// 资源缓存条目：包含数据、MIME 类型、ETag 和访问时间（用于 LRU 驱逐）
struct CacheEntry {
    data: std::sync::Arc<Vec<u8>>,
    mime: &'static str,
    etag: String,
    last_access: std::time::Instant,
}

static RESOURCE_CACHE: LazyLock<DashMap<String, CacheEntry>> = LazyLock::new(DashMap::new);

static CACHE_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);
const MAX_CACHE_SIZE: usize = 50 * 1024 * 1024; // 50MB

fn remember_window_cache_key(window_id: u64, cache_key: impl Into<String>) {
    let cache_key = cache_key.into();
    WINDOW_CACHE_KEYS
        .entry(window_id)
        .or_insert_with(HashSet::new)
        .insert(cache_key);
}

fn remove_cache_entry(cache_key: &str) {
    if let Some((_, entry)) = RESOURCE_CACHE.remove(cache_key) {
        CACHE_TOTAL_SIZE.fetch_sub(entry.data.len(), Ordering::Relaxed);
    }
}

fn cleanup_window_cache(window_id: u64) {
    if let Some((_, cache_keys)) = WINDOW_CACHE_KEYS.remove(&window_id) {
        for cache_key in cache_keys {
            remove_cache_entry(&cache_key);
        }
    }
}

fn insert_resource_cache(
    window_id: Option<u64>,
    cache_key: String,
    data: Arc<Vec<u8>>,
    mime: &'static str,
    etag: String,
) {
    CACHE_TOTAL_SIZE.fetch_add(data.len(), Ordering::Relaxed);
    RESOURCE_CACHE.insert(
        cache_key.clone(),
        CacheEntry {
            data,
            mime,
            etag,
            last_access: std::time::Instant::now(),
        },
    );
    if let Some(window_id) = window_id.filter(|_| cache_key.contains("_wb")) {
        remember_window_cache_key(window_id, cache_key);
    }
}

fn cleanup_window(window_id: u64) {
    crate::app::stream::cleanup_window_streams(window_id);
    cleanup_window_cache(window_id);
    WINDOWS.remove(&window_id);
    WINDOW_PROXIES.remove(&window_id);
    WINDOW_READY.remove(&window_id);
    EVENT_PROXIES.remove(&window_id);
    WEBVIEWS.remove(&window_id);
    if let Ok(mut configs) = WINDOW_CONFIGS.write() {
        configs.remove(&window_id);
    }
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

/// 计算轻量级 ETag（基于文件大小和路径，避免全内容哈希开销）
fn compute_etag(data: &[u8], path: &str) -> String {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash_slice(&(data.len() as u64).to_le_bytes(), &mut hasher);
    std::hash::Hash::hash_slice(path.as_bytes(), &mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 解析 Range 头，返回 (start, end) 字节范围
fn parse_range_header(range_header: &str, total_len: u64) -> Option<(u64, u64)> {
    // 格式: "bytes=start-end" 或 "bytes=start-"
    let range = range_header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let start: u64 = parts[0].parse().ok()?;
    let end = if parts[1].is_empty() {
        total_len - 1
    } else {
        let e: u64 = parts[1].parse().ok()?;
        e.min(total_len - 1)
    };
    if start > end {
        return None;
    }
    Some((start, end))
}

/// 当缓存超过上限时，按 LRU 策略清理：移除最久未访问的条目，直到总大小降到阈值以下
fn evict_cache_if_needed() {
    if CACHE_TOTAL_SIZE.load(Ordering::Relaxed) <= MAX_CACHE_SIZE {
        return;
    }
    // 收集所有条目的 (key, last_access, data_len)，按 last_access 排序
    let mut entries: Vec<(String, std::time::Instant, usize)> = RESOURCE_CACHE
        .iter()
        .map(|r| (r.key().clone(), r.value().last_access, r.value().data.len()))
        .collect();
    entries.sort_by_key(|(_, t, _)| *t);

    // 从最旧的开始删除，直到总大小降到 MAX_CACHE_SIZE / 2
    let target = MAX_CACHE_SIZE / 2;
    for (key, _, _len) in entries {
        if CACHE_TOTAL_SIZE.load(Ordering::Relaxed) <= target {
            break;
        }
        if let Some((_, entry)) = RESOURCE_CACHE.remove(&key) {
            CACHE_TOTAL_SIZE.fetch_sub(entry.data.len(), Ordering::Relaxed);
        }
    }
}

/// 向指定窗口发送 JavaScript 消息（通过事件循环）
pub fn send_script_to_window(window_id: u64, script: std::sync::Arc<String>) -> bool {
    if let Some(proxy) = EVENT_PROXIES.get(&window_id) {
        proxy
            .send_event(UserEvent::EvaluateScript { window_id, script })
            .is_ok()
    } else {
        false
    }
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
        .with_decorations(config.show_title_bar)
        .with_resizable(config.enable_resizable)
        .with_min_inner_size(LogicalSize::new(400u32, 300u32))
        .with_transparent(true)
        .with_undecorated_shadow(false)
        .with_visible(true);

    #[cfg(not(target_os = "windows"))]
    let window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_window_icon(generate_win_icon(config.icon_path.clone()))
        .with_decorations(config.show_title_bar)
        .with_resizable(config.enable_resizable)
        .with_min_inner_size(LogicalSize::new(400u32, 300u32))
        .with_transparent(true);

    let window = match window_builder.build(event_loop) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[pywebron] 窗口创建失败: {:?}", e);
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

    let webview = {
        let _webview_lock = WEBVIEW_CREATE_LOCK.lock().unwrap();

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

        let window_config_json = serde_json::json!({
            "window_id": id_clone,
            "title": config.title,
            "width": config.width,
            "height": config.height,
            "show_title_bar": config.show_title_bar,
            "window_radius": config.window_radius,
            "enable_resizable": config.enable_resizable,
            "enable_devtools": config.enable_devtools,
            "icon_path": config.icon_path
        });

        let initialization_script = format!(
            "window.pywebron={};{}",
            serde_json::to_string(&window_config_json).unwrap_or_default(),
            load_js_api()
        );

        // 存储 dist 路径用于自定义协议处理
        // dist 模式：指向 dist 目录；html 模式：指向 html 文件所在目录；link 模式：优先从 icon_path 推导，否则使用当前工作目录
        let dist_path_for_protocol = if config.dist_content.is_some() {
            let dist_path = std::path::Path::new(config.dist_content.as_ref().unwrap());
            if dist_path.is_absolute() {
                dist_path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(dist_path)
            }
        } else if config.html_content.is_some() {
            let html_path = std::path::Path::new(config.html_content.as_ref().unwrap());
            let absolute_path = if html_path.is_absolute() {
                html_path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(html_path)
            };
            absolute_path
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .to_path_buf()
        } else if !config.icon_path.is_empty() {
            // link_content 模式下，从 icon_path 推导资源目录
            let icon_path = std::path::Path::new(&config.icon_path);
            let absolute_path = if icon_path.is_absolute() {
                icon_path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(icon_path)
            };
            absolute_path
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .to_path_buf()
        } else {
            // 无 icon_path 时回退到当前工作目录
            std::env::current_dir().unwrap_or_default()
        };

        // clone 一份供后续 RESOURCE_CACHE 使用（原值会被 move 进闭包）
        let dist_path_for_cache = dist_path_for_protocol.clone();

        // 预计算 canonical_base（路径穿越防护用），避免每次请求都 canonicalize
        let canonical_base = match dist_path_for_protocol.canonicalize() {
            Ok(p) => Some(p),
            Err(_) => None,
        };
        let allowed_absolute_file = std::path::Path::new(&config.icon_path).canonicalize().ok();
        let allow_absolute_protocol_paths = config.link_content.is_none();

        let builder = WebViewBuilder::new()
            .with_devtools(config.enable_devtools)
            .with_transparent(true)
            .with_background_color((255, 255, 255, 255))
            .with_initialization_script(&initialization_script)
            .with_ipc_handler(move |request| {
                handle_ipc_message(request, window_id_for_ipc, &proxy_for_handler);
            })
            .with_custom_protocol("app".to_string(), move |_id, request| {
                let uri = request.uri().to_string();

                // Windows: http://app.<path> 或 http://app._wb<window_id>/<path>
                // 其他平台: app://<path> 或 app://_wb<window_id>/<path>
                let raw_path = uri
                    .strip_prefix("http://app.")
                    .or_else(|| uri.strip_prefix("https://app."))
                    .or_else(|| uri.strip_prefix("app://"))
                    .unwrap_or("");

                // 剥离查询参数
                let raw_path = raw_path.split('?').next().unwrap_or("");
                // 去除尾部斜杠
                let raw_path = raw_path.trim_end_matches('/');

                // 剥离 _wb<window_id>/ 缓存破坏前缀
                // 格式: _wb123456789/index.html 或 _wb123456789/assets/xxx.js
                let (file_path, wb_id) = if let Some(rest) = raw_path.strip_prefix("_wb") {
                    // 提取数字部分
                    let digits_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len());
                    let id = &rest[..digits_end];
                    let after_digits = rest[digits_end..].trim_start_matches('/');
                    (after_digits, Some(id.to_string()))
                } else {
                    (raw_path, None)
                };

                debug_log(|| format!(
                    "[Protocol] 请求 URI={} | 解析路径={} | wb_id={:?} | dist_base={}",
                    uri,
                    file_path,
                    wb_id,
                    dist_path_for_protocol.display()
                ));

                // 空路径直接返回空，避免无意义的请求
                if file_path.is_empty() {
                    return http::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(std::borrow::Cow::Borrowed(&[][..]))
                        .unwrap();
                }

                // 检测绝对路径（多种格式）
                let abs_candidate = std::path::Path::new(file_path);

                // 格式1：标准 Windows 绝对路径 C:/xxx 或 C:\xxx 或 Unix /xxx
                let is_standard_abs = abs_candidate.is_absolute();

                // 格式2：无冒号的 Windows 路径 d/works/... （JS resolveAssetUrl 可能去掉冒号）
                // 模式：<单个字母>/<非空内容>
                let missing_colon_abs = !is_standard_abs
                    && !file_path.is_empty()
                    && file_path.len() > 2
                    && file_path.as_bytes()[0].is_ascii_alphabetic()
                    && file_path.as_bytes()[1] == b'/'
                    && file_path.contains('/');

                // 尝试重建为标准绝对路径 <letter>:/<rest>
                let reconstructed_path = if missing_colon_abs {
                    let letter = file_path.as_bytes()[0];
                    let rest = &file_path[2..]; // 跳过 "d/"
                    Some(format!("{}:/{}", letter as char, rest))
                } else {
                    None
                };

                // 确定最终使用哪条路径
                let resolved_path = if is_standard_abs {
                    std::path::PathBuf::from(file_path)
                } else if let Some(ref reconstructed) = reconstructed_path {
                    // 验证重建后的路径是否存在，如果存在则用它
                    if std::path::Path::new(reconstructed).exists() {
                        debug_log(|| format!(
                            "[Protocol] 检测到无冒号路径，已重建为: {}",
                            reconstructed
                        ));
                        std::path::PathBuf::from(reconstructed)
                    } else {
                        // 不存在则回退到相对路径拼接
                        dist_path_for_protocol.join(file_path)
                    }
                } else {
                    dist_path_for_protocol.join(file_path)
                };

                let full_path = resolved_path;
                let direct_cache_key = full_path.to_string_lossy().to_string();
                // 带 _wb 前缀的缓存 key（用于 HTML 等每个窗口独立缓存的资源）
                let wb_cache_key = wb_id
                    .as_ref()
                    .map(|id| format!("{}//_wb{}", direct_cache_key, id));
                debug_log(|| format!(
                    "[Protocol] 拼接完整路径={} | 缓存key={} | wb_cachekey={:?}",
                    full_path.display(),
                    direct_cache_key,
                    wb_cache_key
                ));

                let is_absolute_request = is_standard_abs || missing_colon_abs;
                let canonical_full_for_policy = full_path.canonicalize().ok();
                if is_absolute_request && !allow_absolute_protocol_paths {
                    let allowed = match (&canonical_full_for_policy, &allowed_absolute_file) {
                        (Some(full), Some(allowed_file)) => full == allowed_file,
                        _ => false,
                    };

                    if !allowed {
                        debug_log(|| format!("[Protocol] 绝对路径访问被拒绝: {}", full_path.display()));
                        return http::Response::builder()
                            .status(403)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Borrowed(&b"Forbidden"[..]))
                            .unwrap();
                    }
                }

                // 先尝试从缓存直接命中（处理已预缓存的资源）
                // 优先查找带 _wb 前缀的 key（HTML 每个窗口独立缓存），再查找通用 key
                let cache_lookup_key = if let Some(ref wb_key) = wb_cache_key {
                    if RESOURCE_CACHE.contains_key(wb_key) {
                        Some(wb_key.clone())
                    } else if RESOURCE_CACHE.contains_key(&direct_cache_key) {
                        Some(direct_cache_key.clone())
                    } else {
                        None
                    }
                } else {
                    if RESOURCE_CACHE.contains_key(&direct_cache_key) {
                        Some(direct_cache_key.clone())
                    } else {
                        None
                    }
                };

                if let Some(used_key) = cache_lookup_key {
                    if let Some(mut entry) = RESOURCE_CACHE.get_mut(&used_key) {
                        entry.last_access = std::time::Instant::now();
                        let data = entry.data.clone();
                        let mime = entry.mime;
                        let etag = entry.etag.clone();
                        debug_log(|| format!(
                            "[Protocol] 直接缓存命中: {} | mime={} | size={}",
                            used_key,
                            mime,
                            data.len()
                        ));

                        let if_none_match = request
                            .headers()
                            .get("if-none-match")
                            .and_then(|v| v.to_str().ok());
                        if if_none_match == Some(etag.as_str()) {
                            return http::Response::builder()
                                .status(304)
                                .header("ETag", &etag)
                                .header("Access-Control-Allow-Origin", "*")
                                .body(std::borrow::Cow::Borrowed(&b""[..]))
                                .unwrap();
                        }

                        return http::Response::builder()
                            .status(200)
                            .header("Content-Type", mime)
                            .header("ETag", &etag)
                            .header("Cache-Control", "public, max-age=31536000")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Owned(data.to_vec()))
                            .unwrap();
                    }
                }

                debug_log(|| "[Protocol] 未命中直接缓存，尝试磁盘查找...".to_string());

                // 路径穿越防护：使用预计算的 canonical_base
                let canonical_base = match &canonical_base {
                    Some(p) => p,
                    None => {
                        return http::Response::builder()
                            .status(500)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Borrowed(&b"Internal Server Error"[..]))
                            .unwrap();
                    }
                };
                let canonical_full = if let Some(path) = canonical_full_for_policy.clone() {
                    path
                } else {
                    match full_path.canonicalize() {
                        Ok(path) => path,
                        Err(_) => {
                            eprintln!("[Protocol] ❌ canonicalize 失败: {}", full_path.display());
                            // 资源不存在，返回 200 + 空（避免控制台 404 报错）
                            return http::Response::builder()
                                .status(200)
                                .header("Content-Type", "text/plain")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(std::borrow::Cow::Borrowed(&[][..]))
                                .unwrap();
                        }
                    }
                };
                eprintln!(
                    "[Protocol] canonicalize 成功: {} | base={} | is_abs_path={}",
                    canonical_full.display(),
                    canonical_base.display(),
                    is_absolute_request
                );
                // 路径穿越防护：仅对相对路径资源检查
                // 绝对路径（用户显式指定的外部资源如图标）跳过此检查
                if !is_absolute_request && !canonical_full.starts_with(canonical_base) {
                    eprintln!(
                        "[Protocol] 🚫 路径穿越拦截: {} 不在 {} 内",
                        canonical_full.display(),
                        canonical_base.display()
                    );
                    return http::Response::builder()
                        .status(403)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(std::borrow::Cow::Borrowed(&b"Forbidden"[..]))
                        .unwrap();
                }

                let cache_key = canonical_full.to_string_lossy().to_string();

                // 检查 If-None-Match / ETag（304 Not Modified）
                let if_none_match = request
                    .headers()
                    .get("if-none-match")
                    .and_then(|v| v.to_str().ok());
                if let Some(mut entry) = RESOURCE_CACHE.get_mut(&cache_key) {
                    entry.last_access = std::time::Instant::now();
                    eprintln!("[Protocol] ✅ canonical 缓存命中(304检查): {}", cache_key);
                    if if_none_match == Some(entry.etag.as_str()) {
                        return http::Response::builder()
                            .status(304)
                            .header("ETag", &entry.etag)
                            .header("Cache-Control", "public, max-age=31536000")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Borrowed(&b""[..]))
                            .unwrap();
                    }
                }

                // 缓存命中
                if let Some(mut entry) = RESOURCE_CACHE.get_mut(&cache_key) {
                    entry.last_access = std::time::Instant::now();
                    let data = entry.data.clone(); // Arc clone，零拷贝
                    let mime = entry.mime;
                    let etag = entry.etag.clone();
                    eprintln!(
                        "[Protocol] ✅ canonical 缓存命中(返回内容): {} | mime={} | size={}",
                        cache_key,
                        mime,
                        data.len()
                    );

                    // 检查 Range 请求
                    let range_header = request.headers().get("range").and_then(|v| v.to_str().ok());
                    if let Some(range) = range_header {
                        if let Some((start, end)) = parse_range_header(range, data.len() as u64) {
                            let sliced = data[start as usize..=end as usize].to_vec();
                            return http::Response::builder()
                                .status(206)
                                .header("Content-Type", mime)
                                .header(
                                    "Content-Range",
                                    format!("bytes {}-{}/{}", start, end, data.len()),
                                )
                                .header("Content-Length", sliced.len())
                                .header("ETag", &etag)
                                .header("Cache-Control", "public, max-age=31536000")
                                .header("Accept-Ranges", "bytes")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(std::borrow::Cow::Owned(sliced))
                                .unwrap();
                        }
                    }

                    return http::Response::builder()
                        .header("Content-Type", mime)
                        .header("Content-Length", data.len())
                        .header("ETag", &etag)
                        .header("Cache-Control", "public, max-age=31536000")
                        .header("Accept-Ranges", "bytes")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(std::borrow::Cow::Owned(
                            std::sync::Arc::try_unwrap(data).unwrap_or_else(|arc| (*arc).clone()),
                        ))
                        .unwrap();
                }

                // 缓存未命中，读取文件
                // 先尝试从 dist 目录读取；失败后回退到绝对路径查找
                // （resolveAssetUrl 只返回文件名如 "pywebron.png"，可能不在 dist 目录内）
                let read_result = std::fs::read(&full_path);
                eprintln!(
                    "[Protocol] ⏳ 尝试磁盘读取: {} | exists={}",
                    full_path.display(),
                    full_path.exists()
                );
                let (data, actual_mime) = match read_result {
                    Ok(data) => {
                        let mime = get_mime_type(&full_path);
                        eprintln!(
                            "[Protocol] ✅ 磁盘读取成功: {} | size={} | mime={}",
                            full_path.display(),
                            data.len(),
                            mime
                        );
                        (data, mime)
                    }
                    Err(ref e) => {
                        eprintln!(
                            "[Protocol] ❌ 磁盘读取失败: {} | 错误: {}",
                            full_path.display(),
                            e
                        );
                        // 回退：将请求路径视为绝对路径尝试读取
                        let abs_candidate = std::path::Path::new(file_path);
                        if abs_candidate.is_absolute()
                            || (!file_path.contains('/') && !file_path.contains('\\'))
                        {
                            // 纯文件名（如 "pywebron.png"）：无法直接定位，跳过
                            // 绝对路径但不在 dist 内：直接读取
                            if abs_candidate.is_absolute() && abs_candidate.exists() {
                                match std::fs::read(abs_candidate) {
                                    Ok(d) => {
                                        eprintln!(
                                            "[Protocol] ✅ 回退绝对路径成功: {} | size={}",
                                            abs_candidate.display(),
                                            d.len()
                                        );
                                        (d, get_mime_type(abs_candidate))
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[Error][Cache] 文件读取失败(回退): {} | 错误: {}",
                                            file_path, e
                                        );
                                        return http::Response::builder()
                                            .status(200)
                                            .header("Content-Type", "text/plain")
                                            .header("Access-Control-Allow-Origin", "*")
                                            .body(std::borrow::Cow::Borrowed(&[][..]))
                                            .unwrap();
                                    }
                                }
                            } else {
                                eprintln!("[Protocol] ⚠️ 纯文件名无法定位: {}", file_path);
                                return http::Response::builder()
                                    .status(200)
                                    .header("Content-Type", "text/plain")
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(std::borrow::Cow::Borrowed(&[][..]))
                                    .unwrap();
                            }
                        } else {
                            eprintln!(
                                "[Error][Cache] 文件读取失败: {} | 错误: {}",
                                file_path,
                                read_result.unwrap_err()
                            );
                            return http::Response::builder()
                                .status(200)
                                .header("Content-Type", "text/plain")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(std::borrow::Cow::Borrowed(&[][..]))
                                .unwrap();
                        }
                    }
                };

                let mime = actual_mime;
                let etag = compute_etag(&data, &cache_key);

                // 大文件不缓存（>5MB），避免内存占用过高
                if data.len() < 5 * 1024 * 1024 {
                    insert_resource_cache(
                        None,
                        cache_key.clone(),
                        Arc::new(data.clone()),
                        mime,
                        etag.clone(),
                    );
                    eprintln!(
                        "[Protocol] 📦 已写入缓存: {} | size={}",
                        cache_key,
                        data.len()
                    );
                    evict_cache_if_needed();
                }

                // 检查 Range 请求
                let range_header = request.headers().get("range").and_then(|v| v.to_str().ok());
                if let Some(range) = range_header {
                    if let Some((start, end)) = parse_range_header(range, data.len() as u64) {
                        let sliced = data[start as usize..=end as usize].to_vec();
                        return http::Response::builder()
                            .status(206)
                            .header("Content-Type", mime)
                            .header(
                                "Content-Range",
                                format!("bytes {}-{}/{}", start, end, data.len()),
                            )
                            .header("Content-Length", sliced.len())
                            .header("ETag", &etag)
                            .header("Cache-Control", "public, max-age=31536000")
                            .header("Accept-Ranges", "bytes")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Owned(sliced))
                            .unwrap();
                    }
                }

                http::Response::builder()
                    .header("Content-Type", mime)
                    .header("Content-Length", data.len())
                    .header("ETag", &etag)
                    .header("Cache-Control", "public, max-age=31536000")
                    .header("Accept-Ranges", "bytes")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(std::borrow::Cow::Owned(data))
                    .unwrap()
            });

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
                let html_content = match std::fs::read_to_string(&resolved_content) {
                    Ok(html) => html,
                    Err(_) => {
                        eprintln!("[Error] 读取 HTML 文件失败：{}", resolved_content);
                        return;
                    }
                };

                // 将路径改为 app:// 协议，与 dist 模式一致
                let converted = html_content
                    .replace("href=\"/", "href=\"app://")
                    .replace("src=\"/", "src=\"app://")
                    .replace("href='/", "href='app://")
                    .replace("src='/", "src='app://");

                // 将 HTML 内容存入资源缓存，通过自定义协议提供
                let cache_key = dist_path_for_cache
                    .join("index.html")
                    .to_string_lossy()
                    .to_string();
                let html_bytes = converted.as_bytes().to_vec();
                insert_resource_cache(
                    Some(window_id),
                    cache_key,
                    Arc::new(html_bytes),
                    "text/html",
                    compute_etag(converted.as_bytes(), "index.html"),
                );

                builder
                    .with_url(&format!("app://_wb{}/index.html", window_id))
                    .build_gtk(vbox)
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
                        let wb_prefix = format!("_wb{}/", window_id);
                        let mut converted =
                            html.replace("href=\"/", &format!("href=\"app://{}", wb_prefix));
                        converted =
                            converted.replace("src=\"/", &format!("src=\"app://{}", wb_prefix));
                        converted =
                            converted.replace("href='/", &format!("href='app://{}", wb_prefix));
                        converted =
                            converted.replace("src='/", &format!("src='app://{}", wb_prefix));

                        converted
                    }
                    Err(e) => {
                        eprintln!("[Error] 读取 dist/index.html 失败：{}", e);
                        return;
                    }
                };

                // 将转换后的 HTML 存入资源缓存（key 包含 window_id，因为每个窗口 HTML 内容不同）
                // （与 html_file_path 模式一致，确保页面有正确的 origin，
                //  支持 ES Module / crossorigin 等特性）
                let cache_key = format!(
                    "{}//_wb{}",
                    dist_path_for_cache.join("index.html").to_string_lossy(),
                    window_id
                );
                let html_bytes = html_content.as_bytes().to_vec();
                insert_resource_cache(
                    Some(window_id),
                    cache_key,
                    Arc::new(html_bytes),
                    "text/html",
                    compute_etag(html_content.as_bytes(), "index.html"),
                );

                builder
                    .with_url(&format!("app://_wb{}/index.html", window_id))
                    .build_gtk(vbox)
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
            let res = if is_url {
                builder.with_url(&resolved_content).build(&window)
            } else if is_file_path {
                let html_content = match std::fs::read_to_string(&resolved_content) {
                    Ok(html) => html,
                    Err(e) => {
                        eprintln!("[Error] 读取 HTML 文件失败：{}", e);
                        return;
                    }
                };

                // 将路径改为 app:// 协议，与 dist 模式一致
                // 使用 _wb<window_id>/ 前缀确保每个窗口的 URL 不同，避免浏览器缓存共享
                #[cfg(target_os = "windows")]
                let converted = {
                    let wb_prefix = format!("_wb{}/", window_id);
                    html_content
                        .replace("href=\"/", &format!("href=\"http://app.{}", wb_prefix))
                        .replace("src=\"/", &format!("src=\"http://app.{}", wb_prefix))
                        .replace("href='/", &format!("href='http://app.{}", wb_prefix))
                        .replace("src='/", &format!("src='http://app.{}", wb_prefix))
                };

                #[cfg(not(target_os = "windows"))]
                let converted = {
                    let wb_prefix = format!("_wb{}/", window_id);
                    html_content
                        .replace("href=\"/", &format!("href=\"app://{}", wb_prefix))
                        .replace("src=\"/", &format!("src=\"app://{}", wb_prefix))
                        .replace("href='/", &format!("href='app://{}", wb_prefix))
                        .replace("src='/", &format!("src='app://{}", wb_prefix))
                };

                // 将 HTML 内容存入资源缓存（key 包含 window_id，因为每个窗口 HTML 内容不同）
                let cache_key = format!(
                    "{}//_wb{}",
                    dist_path_for_cache.join("index.html").to_string_lossy(),
                    window_id
                );
                let html_bytes = converted.as_bytes().to_vec();
                insert_resource_cache(
                    Some(window_id),
                    cache_key,
                    Arc::new(html_bytes),
                    "text/html",
                    compute_etag(converted.as_bytes(), "index.html"),
                );

                #[cfg(target_os = "windows")]
                let url = format!("http://app._wb{}/index.html", window_id);
                #[cfg(not(target_os = "windows"))]
                let url = format!("app://_wb{}/index.html", window_id);

                builder.with_url(&url).build(&window)
            } else if is_dist {
                let dist_path = std::path::Path::new(&resolved_content);
                let index_html = dist_path.join("index.html");

                if !index_html.exists() {
                    eprintln!("[Error] dist 目录中不存在 index.html：{}", resolved_content);
                    return;
                }

                // 读取 HTML 并将所有路径改为 app 协议
                let html_content = match std::fs::read_to_string(&index_html) {
                    Ok(html) => {
                        #[cfg(target_os = "windows")]
                        let converted = {
                            // 使用 _wb<window_id>/ 前缀确保每个窗口的 URL 不同，避免浏览器缓存共享
                            let wb_prefix = format!("_wb{}/", window_id);
                            html.replace("href=\"/", &format!("href=\"http://app.{}", wb_prefix))
                                .replace("src=\"/", &format!("src=\"http://app.{}", wb_prefix))
                                .replace("href='/", &format!("href='http://app.{}", wb_prefix))
                                .replace("src='/", &format!("src='http://app.{}", wb_prefix))
                        };

                        #[cfg(not(target_os = "windows"))]
                        let converted = {
                            let wb_prefix = format!("_wb{}/", window_id);
                            html.replace("href=\"/", &format!("href=\"app://{}", wb_prefix))
                                .replace("src=\"/", &format!("src=\"app://{}", wb_prefix))
                                .replace("href='/", &format!("href='app://{}", wb_prefix))
                                .replace("src='/", &format!("src='app://{}", wb_prefix))
                        };

                        eprintln!(
                            "[Window-{}] 已转换 HTML，添加缓存破坏前缀 _wb{}",
                            window_id, window_id
                        );

                        converted
                    }
                    Err(e) => {
                        eprintln!("[Error] 读取 dist/index.html 失败：{}", e);
                        return;
                    }
                };

                // 将转换后的 HTML 存入资源缓存（key 包含 window_id，因为每个窗口 HTML 内容不同）
                // （与 html_file_path 模式一致，确保页面有正确的 origin，
                //  支持 ES Module / crossorigin 等特性）
                let cache_key = format!(
                    "{}//_wb{}",
                    dist_path_for_cache.join("index.html").to_string_lossy(),
                    window_id
                );
                let html_bytes = html_content.as_bytes().to_vec();
                insert_resource_cache(
                    Some(window_id),
                    cache_key,
                    Arc::new(html_bytes),
                    "text/html",
                    compute_etag(html_content.as_bytes(), "index.html"),
                );

                #[cfg(target_os = "windows")]
                let build_result = builder
                    .with_url(&format!("http://app._wb{}/index.html", window_id))
                    .build(&window);

                #[cfg(not(target_os = "windows"))]
                let build_result = builder
                    .with_url(&format!("app://_wb{}/index.html", window_id))
                    .build_gtk(vbox);

                build_result
            } else {
                builder
                    .with_html("<html><body>No content specified</body></html>")
                    .build(&window)
            };
            res
        };

        match result {
            Ok(wv) => WebViewWrapper(std::sync::Arc::new(Mutex::new(Some(wv)))),
            Err(_e) => {
                return;
            }
        }
    };

    #[cfg(target_os = "windows")]
    if !hwnd.is_null() {
        if config.enable_resizable && !config.show_title_bar {
            crate::utils::make_window_frameless_but_resizable(windows::Win32::Foundation::HWND(
                hwnd,
            ));
        } else {
            // skip make_window_frameless_but_resizable
        }

        if config.show_title_bar {
            let corner_pref = match config.dwm_corner {
                1 => crate::configs::WindowCorners::DoNotRound,
                2 => crate::configs::WindowCorners::Round,
                3 => crate::configs::WindowCorners::RoundSmall,
                _ => crate::configs::WindowCorners::Default,
            };
            match crate::utils::set_window_corner(
                windows::Win32::Foundation::HWND(hwnd),
                corner_pref,
            ) {
                Ok(_) => {}
                Err(_) => {}
            }
        } else {
            // 无标题栏时，禁用 DWM 圆角
            // 不使用 SetWindowRgn（会产生锯齿），而是依赖 CSS border-radius + 透明窗口
            match crate::utils::set_window_corner(
                windows::Win32::Foundation::HWND(hwnd),
                crate::configs::WindowCorners::DoNotRound,
            ) {
                Ok(_) => {}
                Err(_) => {}
            }

            // 注意：不再使用 set_window_rounded_region，因为它会产生锯齿
            // 圆角效果完全由 CSS 实现
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
    pub html_content: Option<String>,
    pub link_content: Option<String>,
    pub dist_content: Option<String>,
    pub icon_path: String,
    pub show_title_bar: bool,
    pub window_radius: u32,
    pub enable_resizable: bool,
    pub enable_devtools: bool,
    pub dwm_corner: u32,
    pub is_main: bool,
}

fn create_window(window_id: u64, config: WindowConfig) -> PyResult<u64> {
    // 检查是否尝试创建第二个主窗口
    if config.is_main {
        let mut main_id = MAIN_WINDOW_ID.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("无法获取主窗口锁: {}", e))
        })?;

        if main_id.is_some() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "主窗口已存在，不能创建第二个主窗口",
            ));
        }

        // 设置主窗口 ID
        *main_id = Some(window_id);
    }

    // 存储配置，让主事件循环可以创建窗口
    if let Ok(mut configs) = PENDING_WINDOWS.lock() {
        configs.insert(window_id, config);
    }

    WINDOW_READY.insert(window_id, false);

    // 唤醒事件循环，让它立即处理待创建的窗口
    if let Ok(pending_configs) = PENDING_WINDOWS.lock() {
        if !pending_configs.is_empty() {
            drop(pending_configs);
            if let Ok(proxy_guard) = MAIN_EVENT_PROXY.lock() {
                if let Some(proxy) = proxy_guard.as_ref() {
                    proxy.send_event(UserEvent::WakeUp).ok();
                }
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
    let body = request.body();
    debug_log(|| format!("[IPC] {}", &body[..body.len().min(200)]));

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
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
                            script: std::sync::Arc::new(js_code),
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
                            script: std::sync::Arc::new(js_code),
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
                            script: std::sync::Arc::new(js_code),
                        });
                        return;
                    }

                    // 提交到 invoke 线程池，结果自动通过 proxy 发回前端
                    if let Err(err) = crate::app::invoke::submit_invoke_ipc(
                        handle_id,
                        window_id,
                        request_id,
                        payload,
                        proxy.clone(),
                    ) {
                        let response = serde_json::json!({
                            "window_id": window_id,
                            "handle_id": "",
                            "handle_type": "invoke",
                            "request_id": null,
                            "payload": {"code": 503, "mssg": err, "data": null}
                        });
                        let js_code = format!(
                            "window.__pywebron_dispatch({})",
                            serde_json::to_string(&response).unwrap_or_default()
                        );
                        let _ = proxy.send_event(UserEvent::EvaluateScript {
                            window_id,
                            script: Arc::new(js_code),
                        });
                    }
                }
                "stream" => {
                    use crate::app::stream::{
                        is_handler_active, is_stream_active, push_stream_data,
                        register_stream_window, send_history_to_window,
                    };

                    // Stream 处理：区分"启动"和"数据"消息
                    if is_stream_active(&handle_id, window_id) {
                        // 该窗口已订阅：直接投递数据到队列
                        push_stream_data(&handle_id, window_id, payload);
                    } else if is_handler_active(&handle_id) {
                        // handler 已活跃：注册新窗口订阅、发送历史消息、推送数据
                        register_stream_window(&handle_id, window_id);
                        send_history_to_window(&handle_id, window_id);
                        push_stream_data(&handle_id, window_id, payload);
                    } else {
                        // handler 未启动：提交到 stream 线程池启动
                        let _ = crate::app::invoke::submit_stream_ipc(
                            handle_id, window_id, request_id, payload,
                        );
                    }
                }
                "stream_close" => {
                    if !handle_id.is_empty() {
                        crate::app::stream::unregister_stream_window(&handle_id, window_id);
                    }
                }
                _ => {}
            }
        }
    }
}

#[pyfunction(name = "rust_init")]
#[pyo3(signature = (prewarm_webview=false))]
pub fn init(py: Python<'_>, prewarm_webview: bool) -> PyResult<()> {
    #[cfg(target_os = "windows")]
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "00000000");

    // `warm_python_runtime` spawns a dedicated Python loop thread and waits for it
    // to attach. Detach the current thread first so the spawned thread can attach
    // instead of deadlocking against this Python->Rust call.
    py.detach(crate::app::invoke::warm_python_runtime)?;

    #[cfg(target_os = "windows")]
    if prewarm_webview {
        prewarm_webview2();
    }

    #[cfg(not(target_os = "windows"))]
    let _ = prewarm_webview;

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
#[pyo3(signature = (title, width, height, html_content, link_content, dist_content, icon_path, show_title_bar, window_radius, enable_resizable, enable_devtools, window_id, dwm_corner=0, is_main=false))]
pub fn register_window(
    title: String,
    width: u32,
    height: u32,
    html_content: Option<String>,
    link_content: Option<String>,
    dist_content: Option<String>,
    icon_path: String,
    show_title_bar: bool,
    window_radius: u32,
    enable_resizable: bool,
    enable_devtools: bool,
    window_id: u64,
    dwm_corner: u32,
    is_main: bool,
) -> PyResult<u64> {
    let config = WindowConfig {
        title: title.clone(),
        width,
        height,
        html_content,
        link_content,
        dist_content,
        icon_path,
        show_title_bar,
        window_radius,
        enable_resizable,
        enable_devtools,
        dwm_corner,
        is_main,
    };

    // 创建窗口并返回 window_id
    let window_id = create_window(window_id, config.clone())?;

    // 存储配置供 get_windows() 查询
    if let Ok(mut configs) = WINDOW_CONFIGS.write() {
        configs.insert(window_id, config);
    }

    Ok(window_id)
}

use std::sync::RwLock;

static WINDOW_CONFIGS: LazyLock<RwLock<HashMap<u64, WindowConfig>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

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

    // 关键：释放 GIL，允许其他线程（invoke/stream 线程池）访问 Python
    // event_loop.run() 是阻塞调用，如果不释放 GIL，其他线程的 Python::attach() 会死锁
    // 使用 PyEval_SaveThread/PyEval_RestoreThread 手动管理 GIL
    let _thread_state = unsafe { PyEval_SaveThread() };

    event_loop.run(move |event, event_loop, flow| {
        *flow = ControlFlow::Wait;

        // 处理窗口关闭事件
        match &event {
            Event::UserEvent(UserEvent::WakeUp) => {
                if let Ok(mut pending) = PENDING_WINDOWS.lock() {
                    for (window_id, config) in pending.drain() {
                        create_window_in_event_loop(&config, window_id, &proxy, event_loop);
                    }
                }
            }
            Event::NewEvents(_) => {
                // NewEvents 时也检查待创建窗口（处理事件循环自然唤醒的情况）
                if let Ok(mut pending) = PENDING_WINDOWS.lock() {
                    if !pending.is_empty() {
                        for (window_id, config) in pending.drain() {
                            create_window_in_event_loop(&config, window_id, &proxy, event_loop);
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::CloseWindow(window_id)) => {
                // 检查是否是主窗口
                let is_main_window = if let Ok(main_id) = MAIN_WINDOW_ID.lock() {
                    main_id.as_ref() == Some(window_id)
                } else {
                    false
                };

                if is_main_window {
                    // 关闭主窗口：清理所有窗口并退出应用
                    let all_window_ids: Vec<u64> =
                        WINDOWS.iter().map(|entry| *entry.key()).collect();
                    for id in all_window_ids {
                        cleanup_window(id);
                    }
                    // 清除主窗口 ID
                    if let Ok(mut main_id) = MAIN_WINDOW_ID.lock() {
                        *main_id = None;
                    }
                    *flow = ControlFlow::Exit;
                } else {
                    // 关闭普通窗口：只清理该窗口
                    cleanup_window(*window_id);
                    // 如果所有窗口都关闭了，退出应用
                    if WINDOWS.is_empty() {
                        *flow = ControlFlow::Exit;
                    }
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
                    // 检查是否是主窗口
                    let is_main_window = if let Ok(main_id) = MAIN_WINDOW_ID.lock() {
                        main_id.as_ref() == Some(&id)
                    } else {
                        false
                    };

                    if is_main_window {
                        // 关闭主窗口：清理所有窗口并退出应用
                        let all_window_ids: Vec<u64> =
                            WINDOWS.iter().map(|entry| *entry.key()).collect();
                        for window_id in all_window_ids {
                            cleanup_window(window_id);
                        }
                        // 清除主窗口 ID
                        if let Ok(mut main_id) = MAIN_WINDOW_ID.lock() {
                            *main_id = None;
                        }
                        *flow = ControlFlow::Exit;
                    } else {
                        // 关闭普通窗口：只清理该窗口
                        cleanup_window(id);
                        // 如果所有窗口都关闭了，退出应用
                        if WINDOWS.is_empty() {
                            *flow = ControlFlow::Exit;
                        }
                    }
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
            script: std::sync::Arc::new(js_code),
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
            script: std::sync::Arc::new(js_code),
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
            script: std::sync::Arc::new(js_code),
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
    cleanup_window(id);
    Ok(true)
}

#[pyfunction(name = "rust_get_windows")]
pub fn get_windows(py: Python<'_>) -> PyResult<Bound<'_, pyo3::types::PyDict>> {
    let result_dict = pyo3::types::PyDict::new(py);

    if let Ok(configs) = WINDOW_CONFIGS.read() {
        let live_window_ids: Vec<u64> = EVENT_PROXIES.iter().map(|entry| *entry.key()).collect();
        for window_id in live_window_ids {
            if let Some(config) = configs.get(&window_id) {
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

                result_dict.set_item(window_id, window_dict)?;
            }
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
                Ok(_bytes_copied) => {
                    // 如果需要删除源文件
                    if is_del_source_file {
                        let _ = fs::remove_file(&source_path);
                    }

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

use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::OnceLock;

pub fn load_js_api() -> String {
    let js_content = include_str!("../../assets/pywebron.js");
    js_content.to_string()
}

// === Stream 接收数据队列（有界队列防止 OOM，handle_id:window_id -> Queue） ===
type RecvQueues = HashMap<String, crossbeam::queue::ArrayQueue<Value>>;
static RECV_QUEUES: OnceLock<Arc<RwLock<RecvQueues>>> = OnceLock::new();

// === 活跃 Handler 跟踪（handle_id -> 是否有正在运行的 handler） ===
static ACTIVE_HANDLERS: OnceLock<Arc<RwLock<HashSet<String>>>> = OnceLock::new();

#[inline]
fn get_active_handlers() -> &'static Arc<RwLock<HashSet<String>>> {
    ACTIVE_HANDLERS.get_or_init(|| Arc::new(RwLock::new(HashSet::with_capacity(16))))
}

/// 检查指定 handle_id 是否有活跃的 handler 正在运行
pub fn is_handler_active(handle_id: &str) -> bool {
    let active = get_active_handlers();
    active.read().contains(handle_id)
}

/// 标记 handler 为活跃状态
#[allow(dead_code)]
fn mark_handler_active(handle_id: &str) {
    let active = get_active_handlers();
    active.write().insert(handle_id.to_string());
}

/// 标记 handler 为非活跃状态（handler 结束时调用）
#[allow(dead_code)]
fn mark_handler_inactive(handle_id: &str) {
    let active = get_active_handlers();
    active.write().remove(handle_id);
}

#[inline]
fn get_recv_queues() -> &'static Arc<RwLock<RecvQueues>> {
    RECV_QUEUES.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

// 每个 handle_id 最多缓存 100 条消息
const RECV_QUEUE_LIMIT: usize = 100;

// === 启动 Stream handle（同步版本，由调用方在线程中调用） ===
#[allow(dead_code)]
pub(crate) fn start_stream_handle(
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Value,
) {
    let start = std::time::Instant::now();

    // 注册订阅关系
    register_stream_subscription(handle_id.clone(), window_id);

    // 创建 recv 队列
    {
        let queue_key = format!("{}:{}", handle_id, window_id);
        let queues = get_recv_queues();
        let mut queues_guard = queues.write();
        queues_guard
            .entry(queue_key)
            .or_insert_with(|| crossbeam::queue::ArrayQueue::new(RECV_QUEUE_LIMIT));
    }

    // 标记 handler 为活跃状态
    mark_handler_active(&handle_id);

    let handle_id_for_cleanup = handle_id.clone();
    let t_gil = std::time::Instant::now();
    let result = Python::attach(|py| -> PyResult<()> {
        let gil_elapsed = t_gil.elapsed();
        if gil_elapsed.as_micros() > 100 {
            eprintln!("[Timing][Rust] 获取 GIL 耗时: {:?}", gil_elapsed);
        }
        let t_import = std::time::Instant::now();
        let configs = py.import("pywebron.configs")?;
        let stream_handles = configs.getattr("STREAM_HANDLES")?;
        let import_elapsed = t_import.elapsed();
        if import_elapsed.as_micros() > 100 {
            eprintln!(
                "[Timing][Rust] 导入 configs + 获取 STREAM_HANDLES 耗时: {:?}",
                import_elapsed
            );
        }

        if let Ok(handler) = stream_handles.get_item(&handle_id) {
            let t_build = std::time::Instant::now();
            let request_obj = serde_json::json!({
                "window_id": window_id,
                "handle_id": &handle_id,
                "request_id": &request_id,
                "payload": payload
            });
            let py_request = pythonize::pythonize(py, &request_obj)?;
            let coroutine = handler.call1((py_request,))?;
            let build_elapsed = t_build.elapsed();
            if build_elapsed.as_micros() > 100 {
                eprintln!(
                    "[Timing][Rust] 构建 request + 创建协程耗时: {:?}",
                    build_elapsed
                );
            }

            let t_run = std::time::Instant::now();
            let asyncio = py.import("asyncio")?;
            asyncio.call_method1("run", (coroutine,))?;
            let run_elapsed = t_run.elapsed();
            if run_elapsed.as_millis() > 10 {
                eprintln!(
                    "[Timing][Rust] asyncio.run() 总耗时: {:?} (含 Python handler 全部执行)",
                    run_elapsed
                );
            }
        }
        Ok(())
    });

    // handler 结束，清理活跃状态
    mark_handler_inactive(&handle_id_for_cleanup);

    if let Err(e) = result {
        eprintln!(
            "[Stream] handler 执行错误: {:?} | handle={}",
            e, handle_id_for_cleanup
        );
    }

    let total = start.elapsed();
    if total.as_millis() > 10 {
        eprintln!("[Timing][Rust] start_stream_handle 总耗时: {:?}", total);
    }
}

// === Stream 接收数据（从任意订阅窗口的队列取消息） ===
#[pyfunction(name = "rust_stream_recv")]
pub fn stream_recv<'py>(py: Python<'py>, handle_id: String) -> PyResult<Bound<'py, PyAny>> {
    let hid = handle_id.clone();
    let future = pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let t = std::time::Instant::now();

        // 获取该 handle 的所有订阅窗口
        let subscriptions = get_stream_subscriptions_storage();
        let subs = subscriptions.read();
        let window_ids: Vec<u64> = subs.get(&hid).cloned().unwrap_or_default();

        if window_ids.is_empty() {
            eprintln!("[Timing][Stream] recv 没有订阅的窗口 | handle={}", hid);
            return Ok(None);
        }

        let queues = get_recv_queues();
        let queues_read = queues.read();
        let t_lock = t.elapsed();

        // 轮询所有订阅窗口的队列，从任意窗口取消息
        let mut result_data = None;
        let mut source_window_id = 0u64;

        for window_id in &window_ids {
            let queue_key = format!("{}:{}", hid, window_id);
            if let Some(queue) = queues_read.get(&queue_key) {
                if let Some(data) = queue.pop() {
                    result_data = Some(data);
                    source_window_id = *window_id;
                    break;
                }
            }
        }

        let result = if let Some(payload) = result_data {
            // 构建完整的协议响应格式
            let response = serde_json::json!({
                "handle_id": hid,
                "window_id": source_window_id,
                "handle_type": "stream",
                "request_id": null,
                "payload": payload
            });

            let t_py = std::time::Instant::now();
            let py_result = Python::attach(|py_inner| {
                pythonize::pythonize(py_inner, &response).map(|obj| obj.unbind())
            });
            let total_elapsed = t.elapsed();
            if total_elapsed.as_micros() > 100 {
                eprintln!(
                    "[Timing][Stream] recv 总耗时: {:?} (锁={:?}, py={:?}) | handle={} | from_wid={}",
                    total_elapsed, t_lock, t_py.elapsed(), hid, source_window_id
                );
            }
            if let Ok(py_obj) = py_result {
                Ok(Some(py_obj))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        };

        result
    })?;

    Ok(future)
}

/// 检查指定 handle_id + window_id 的 stream 是否已激活
pub(crate) fn is_stream_active(handle_id: &str, window_id: u64) -> bool {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id)
        .map_or(false, |ids| ids.contains(&window_id))
}

/// 仅注册窗口订阅关系（不启动 handler），用于已有活跃 handler 时的后续窗口订阅
pub(crate) fn register_stream_window(handle_id: &str, window_id: u64) {
    // 注册订阅关系
    register_stream_subscription(handle_id.to_string(), window_id);

    // 创建 recv 队列
    let queue_key = format!("{}:{}", handle_id, window_id);
    let queues = get_recv_queues();
    let mut queues_guard = queues.write();
    queues_guard
        .entry(queue_key)
        .or_insert_with(|| crossbeam::queue::ArrayQueue::new(RECV_QUEUE_LIMIT));
}

/// 将前端发来的数据推入 recv 队列（供 Python stream.recv() 消费）
/// 消息中会注入 source_window_id 字段，让 Python 端知道消息来源
pub(crate) fn push_stream_data(handle_id: &str, window_id: u64, data: Value) {
    let t = std::time::Instant::now();

    // 在消息中注入来源窗口信息
    let enriched_data = if let Some(obj) = data.as_object() {
        let mut enriched = obj.clone();
        enriched.insert(
            "_source_window_id".to_string(),
            serde_json::json!(window_id),
        );
        serde_json::Value::Object(enriched)
    } else {
        data
    };

    let queue_key = format!("{}:{}", handle_id, window_id);
    let queues = get_recv_queues();
    let mut queues_guard = queues.write();
    let t_lock = t.elapsed();
    let queue = queues_guard
        .entry(queue_key)
        .or_insert_with(|| crossbeam::queue::ArrayQueue::new(RECV_QUEUE_LIMIT));
    let _ = queue.push(enriched_data);
    let t_total = t.elapsed();
    if t_total.as_micros() > 50 {
        eprintln!(
            "[Timing][Stream] push_stream_data 耗时: {:?} (锁={:?}) | handle={} | wid={}",
            t_total, t_lock, handle_id, window_id
        );
    }
}

// === Stream 订阅管理（handle_id -> 订阅信息） ===
type StreamSubscriptions = HashMap<String, Vec<u64>>;
static STREAM_SUBSCRIPTIONS: OnceLock<Arc<parking_lot::RwLock<StreamSubscriptions>>> =
    OnceLock::new();

#[inline]
fn get_stream_subscriptions_storage() -> &'static Arc<parking_lot::RwLock<StreamSubscriptions>> {
    STREAM_SUBSCRIPTIONS
        .get_or_init(|| Arc::new(parking_lot::RwLock::new(HashMap::with_capacity(16))))
}

/// 注册 stream 订阅（handle_id 与 window_id 的绑定关系）
fn register_stream_subscription(handle_id: String, window_id: u64) {
    let subscriptions = get_stream_subscriptions_storage();
    let mut subs = subscriptions.write();
    subs.entry(handle_id)
        .or_insert_with(Vec::new)
        .push(window_id);
}

/// 获取指定 handle_id 的所有订阅窗口
pub(crate) fn get_stream_subscriptions(handle_id: &str) -> Vec<u64> {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id).cloned().unwrap_or_default()
}

/// 清理指定窗口的所有 stream 订阅
#[allow(dead_code)]
fn cleanup_window_streams(window_id: u64) {
    let subscriptions = get_stream_subscriptions_storage();
    let mut subs = subscriptions.write();

    for (_, window_ids) in subs.iter_mut() {
        window_ids.retain(|&id| id != window_id);
    }

    subs.retain(|_, window_ids| !window_ids.is_empty());

    let queues = get_recv_queues();
    let mut queues_guard = queues.write();
    queues_guard.retain(|key: &String, _| !key.ends_with(&format!(":{}", window_id)));
}

// === Stream 发送数据到前端（异步版本，支持 Python await） ===
#[pyfunction(name = "rust_stream_send")]
pub fn stream_send<'py>(
    py: Python<'py>,
    payload: Bound<'_, PyAny>,
    handle_id: String,
    send_mode: String,
    window_ids: Option<Vec<u64>>,
) -> PyResult<Bound<'py, PyAny>> {
    let t_entry = std::time::Instant::now();

    // 将 Python payload 转换为 JSON
    let payload_json: Value =
        Python::attach(|_py| pythonize::depythonize(&payload).unwrap_or(Value::Null));

    // 构建响应消息
    let response = serde_json::json!({
        "handle_id": handle_id,
        "handle_type": "stream",
        "request_id": null,
        "payload": payload_json
    });

    // 构建 JavaScript 代码（使用 Arc 避免多窗口广播时的字符串 clone）
    let js_code = Arc::new(format!(
        "window.__pywebron_dispatch({})",
        serde_json::to_string(&response).unwrap_or_default()
    ));
    let _t_build_done = t_entry.elapsed();

    // 根据发送模式和 window_ids 确定目标窗口
    let target_windows: Vec<u64> = match send_mode.as_str() {
        "broadcast" => crate::app::get_all_window_ids(),
        "multicast" | "unitycast" => window_ids.unwrap_or_default(),
        _ => {
            let subscribed = get_stream_subscriptions(&handle_id);
            if !subscribed.is_empty() {
                subscribed
            } else {
                crate::app::get_all_window_ids()
            }
        }
    };

    // 发送消息到目标窗口（Arc 共享，零拷贝）
    for window_id in target_windows {
        crate::app::send_script_to_window(window_id, (*js_code).clone());
    }

    // 返回一个可 await 的 Future（实际已同步完成，但支持 Python await 语法）
    pyo3_async_runtimes::tokio::future_into_py(py, async move { Ok(()) })
}

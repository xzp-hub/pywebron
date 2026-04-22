use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::OnceLock;

pub fn load_js_api() -> &'static str {
    static JS_API: once_cell::sync::Lazy<String> =
        once_cell::sync::Lazy::new(|| include_str!("../../builtins/pywebron.js").to_string());
    &JS_API
}

// === Stream 接收数据队列（有界队列防止 OOM，handle_id:window_id -> Queue） ===
// 存储 (source_window_id, payload) 元组
type RecvQueues = HashMap<String, crossbeam::queue::ArrayQueue<(u64, Value)>>;
static RECV_QUEUES: OnceLock<Arc<RwLock<RecvQueues>>> = OnceLock::new();

// === 活跃 Handler 跟踪（handle_id -> 是否有正在运行的 handler） ===
static ACTIVE_HANDLERS: OnceLock<Arc<RwLock<HashSet<String>>>> = OnceLock::new();

// === 历史消息存储（handle_id -> messages） ===
type StreamHistory = HashMap<String, Vec<Value>>;
static STREAM_HISTORY: OnceLock<Arc<RwLock<StreamHistory>>> = OnceLock::new();
const HISTORY_LIMIT: usize = 200;

#[inline]
pub(crate) fn get_active_handlers() -> &'static Arc<RwLock<HashSet<String>>> {
    ACTIVE_HANDLERS.get_or_init(|| Arc::new(RwLock::new(HashSet::with_capacity(16))))
}

/// 检查指定 handle_id 是否有活跃的 handler 正在运行
pub fn is_handler_active(handle_id: &str) -> bool {
    let active = get_active_handlers();
    active.read().contains(handle_id)
}

#[inline]
fn get_recv_queues() -> &'static Arc<RwLock<RecvQueues>> {
    RECV_QUEUES.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

#[inline]
fn get_stream_history() -> &'static Arc<RwLock<StreamHistory>> {
    STREAM_HISTORY.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

// 每个 handle_id 最多缓存 100 条消息
const RECV_QUEUE_LIMIT: usize = 100;

// === 历史消息管理 ===

/// 存储消息到历史记录
pub(crate) fn store_stream_history(handle_id: &str, payload: Value) {
    let history = get_stream_history();
    let mut h = history.write();
    let msgs = h.entry(handle_id.to_string()).or_insert_with(Vec::new);
    if msgs.len() >= HISTORY_LIMIT {
        msgs.remove(0);
    }
    msgs.push(payload);
}

/// 向新订阅窗口发送历史消息
pub(crate) fn send_history_to_window(handle_id: &str, window_id: u64) {
    let messages = {
        let history = get_stream_history();
        let h = history.read();
        h.get(handle_id).cloned().unwrap_or_default()
    };

    for payload in messages {
        let response = serde_json::json!({
            "handle_id": handle_id,
            "handle_type": "stream",
            "request_id": null,
            "payload": payload
        });
        let js_code = format!(
            "window.__pywebron_dispatch({})",
            serde_json::to_string(&response).unwrap_or_default()
        );
        crate::app::send_script_to_window(window_id, std::sync::Arc::new(js_code));
    }
}

// === Stream 接收数据（从任意订阅窗口的队列取消息） ===
#[pyfunction(name = "rust_stream_recv")]
pub fn stream_recv<'py>(py: Python<'py>, handle_id: String) -> PyResult<Bound<'py, PyAny>> {
    let hid = handle_id.clone();
    let future = pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let queues = get_recv_queues();
        let queues_read = queues.read();

        // 获取该 handle 的所有订阅窗口
        let subscriptions = get_stream_subscriptions_storage();
        let subs = subscriptions.read();
        let window_ids: Vec<u64> = subs.get(&hid).cloned().unwrap_or_default();

        if window_ids.is_empty() {
            return Ok(None);
        }

        // 轮询所有订阅窗口的队列，从任意窗口取消息
        let mut result_data = None;

        for window_id in &window_ids {
            let queue_key = format!("{}:{}", hid, window_id);
            if let Some(queue) = queues_read.get(&queue_key) {
                if let Some((source_window_id, payload)) = queue.pop() {
                    result_data = Some((source_window_id, payload));
                    break;
                }
            }
        }

        let result = if let Some((source_window_id, payload)) = result_data {
            // 构建完整的协议响应格式（包含 source_window_id）
            let response = serde_json::json!({
                "handle_id": hid,
                "handle_type": "stream",
                "request_id": null,
                "source_window_id": source_window_id,
                "payload": payload
            });

            let py_result = Python::attach(|py_inner| {
                pythonize::pythonize(py_inner, &response).map(|obj| obj.unbind())
            });

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
/// 同时记录来源窗口 ID，以便 handler 可以定向回复
pub(crate) fn push_stream_data(handle_id: &str, window_id: u64, data: Value) {
    let queue_key = format!("{}:{}", handle_id, window_id);
    let queues = get_recv_queues();
    let mut queues_guard = queues.write();

    let queue = queues_guard
        .entry(queue_key)
        .or_insert_with(|| crossbeam::queue::ArrayQueue::new(RECV_QUEUE_LIMIT));
    // 存储 (source_window_id, payload)，source_window_id 就是发送消息的窗口
    let _ = queue.push((window_id, data));
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

/// 注册 stream 订阅（handle_id 与 window_id 的绑定关系，带去重）
fn register_stream_subscription(handle_id: String, window_id: u64) {
    let subscriptions = get_stream_subscriptions_storage();
    let mut subs = subscriptions.write();
    subs.entry(handle_id.clone())
        .or_insert_with(Vec::new);
    // 去重：如果已订阅则不再添加
    let ids = subs.get_mut(&handle_id).unwrap(); // safe: just inserted above if missing
    if !ids.contains(&window_id) {
        ids.push(window_id);
    }
}

/// 获取指定 handle_id 的所有订阅窗口
pub(crate) fn get_stream_subscriptions(handle_id: &str) -> Vec<u64> {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id).cloned().unwrap_or_default()
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

    // 存储到历史消息
    store_stream_history(&handle_id, payload_json);

    // 构建 JavaScript 代码（使用 Arc 避免多窗口广播时的字符串 clone）
    let js_code = Arc::new(format!(
        "window.__pywebron_dispatch({})",
        serde_json::to_string(&response).unwrap_or_default()
    ));
    let _t_build_done = t_entry.elapsed();

    // 根据发送模式和 window_ids 确定目标窗口
    let target_windows: Vec<u64> = match send_mode.as_str() {
        "broadcast" => crate::app::get_all_window_ids(),
        "subscribedcast" => get_stream_subscriptions(&handle_id),
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

    // 发送消息到目标窗口（Arc 共享引用，避免重复 clone 字符串）
    for window_id in target_windows {
        crate::app::send_script_to_window(window_id, std::sync::Arc::clone(&js_code));
    }

    // 返回一个可 await 的 Future（实际已同步完成，但支持 Python await 语法）
    pyo3_async_runtimes::tokio::future_into_py(py, async move { Ok(()) })
}

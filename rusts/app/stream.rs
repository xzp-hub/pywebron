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

// === Stream 鎺ユ敹鏁版嵁闃熷垪锛堟湁鐣岄槦鍒楅槻�?OOM锛宧andle_id:window_id -> Queue�?===
// 瀛樺�?(source_window_id, payload) 鍏冪�?
type RecvQueues = HashMap<String, crossbeam::queue::ArrayQueue<(u64, Value)>>;
static RECV_QUEUES: OnceLock<Arc<RwLock<RecvQueues>>> = OnceLock::new();

// === 娲昏�?Handler 璺熻釜锛坔andle_id -> 鏄惁鏈夋鍦ㄨ繍琛岀�?handler�?===
static ACTIVE_HANDLERS: OnceLock<Arc<RwLock<HashSet<String>>>> = OnceLock::new();

// === 鏈€杩戞秷鎭潵婧愮獥鍙ｏ紙handle_id -> 鏉ユ�?window_id锛夛紝鐢ㄤ簬 UNITYCAST 璺�?===
type LastSourceMap = HashMap<String, u64>;
static LAST_SOURCE_WINDOWS: OnceLock<Arc<RwLock<LastSourceMap>>> = OnceLock::new();

// === 鍘嗗彶娑堟伅瀛樺偍锛坔andle_id -> messages�?===
type StreamHistory = HashMap<String, Vec<Value>>;
static STREAM_HISTORY: OnceLock<Arc<RwLock<StreamHistory>>> = OnceLock::new();
const HISTORY_LIMIT: usize = 200;

#[inline]
pub(crate) fn get_active_handlers() -> &'static Arc<RwLock<HashSet<String>>> {
    ACTIVE_HANDLERS.get_or_init(|| Arc::new(RwLock::new(HashSet::with_capacity(16))))
}

/// 妫€鏌ユ寚�?handle_id 鏄惁鏈夋椿璺冪�?handler 姝ｅ湪杩愯
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

#[inline]
fn get_last_source_windows() -> &'static Arc<RwLock<LastSourceMap>> {
    LAST_SOURCE_WINDOWS.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

// 姣忎�?handle_id 鏈€澶氱紦�?100 鏉℃秷鎭?
const RECV_QUEUE_LIMIT: usize = 100;

// === 鍘嗗彶娑堟伅绠＄�?===

/// 瀛樺偍娑堟伅鍒板巻鍙茶�?
pub(crate) fn store_stream_history(handle_id: &str, payload: Value) {
    let history = get_stream_history();
    let mut h = history.write();
    let msgs = h.entry(handle_id.to_string()).or_insert_with(Vec::new);
    if msgs.len() >= HISTORY_LIMIT {
        msgs.remove(0);
    }
    msgs.push(payload);
}

/// 鍚戞柊璁㈤槄绐楀彛鍙戦€佸巻鍙叉秷鎭?
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

// === Stream 鎺ユ敹鏁版嵁锛堜粠浠绘剰璁㈤槄绐楀彛鐨勯槦鍒楀彇娑堟伅�?===
#[pyfunction(name = "rust_stream_recv")]
pub fn stream_recv<'py>(py: Python<'py>, handle_id: String) -> PyResult<Bound<'py, PyAny>> {
    let hid = handle_id.clone();
    let future = pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let queues = get_recv_queues();
        let queues_read = queues.read();

        // 鑾峰彇璇?handle 鐨勬墍鏈夎闃呯獥鍙?
        let subscriptions = get_stream_subscriptions_storage();
        let subs = subscriptions.read();
        let window_ids: Vec<u64> = subs.get(&hid).cloned().unwrap_or_default();

        if window_ids.is_empty() {
            return Ok(None);
        }

        // 杞鎵€鏈夎闃呯獥鍙ｇ殑闃熷垪锛屼粠浠绘剰绐楀彛鍙栨秷�?
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
            // 鏋勫缓瀹屾暣鐨勫崗璁搷搴旀牸寮忥紙鍖呭�?source_window_id�?
            let response = serde_json::json!({
                "handle_id": hid,
                "handle_type": "stream",
                "request_id": null,
                "window_id": source_window_id,
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

/// 妫€鏌ユ寚�?handle_id + window_id �?stream 鏄惁宸叉縺�?
pub(crate) fn is_stream_active(handle_id: &str, window_id: u64) -> bool {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id)
        .map_or(false, |ids| ids.contains(&window_id))
}

/// 浠呮敞鍐岀獥鍙ｈ闃呭叧绯伙紙涓嶅惎鍔?handler锛夛紝鐢ㄤ簬宸叉湁娲昏穬 handler 鏃剁殑鍚庣画绐楀彛璁㈤槄
pub(crate) fn register_stream_window(handle_id: &str, window_id: u64) {
    // 娉ㄥ唽璁㈤槄鍏崇�?
    register_stream_subscription(handle_id.to_string(), window_id);

    // 鍒涘�?recv 闃熷�?
    let queue_key = format!("{}:{}", handle_id, window_id);
    let queues = get_recv_queues();
    let mut queues_guard = queues.write();
    queues_guard
        .entry(queue_key)
        .or_insert_with(|| crossbeam::queue::ArrayQueue::new(RECV_QUEUE_LIMIT));
}

/// 灏嗗墠绔彂鏉ョ殑鏁版嵁鎺ㄥ�?recv 闃熷垪锛堜緵 Python stream.recv() 娑堣垂锛?
/// 鍚屾椂璁板綍鏉ユ簮绐楀�?ID锛屼互渚?handler 鍙互瀹氬悜鍥炲
pub(crate) fn push_stream_data(handle_id: &str, window_id: u64, data: Value) {
    let queue_key = format!("{}:{}", handle_id, window_id);
    let queues = get_recv_queues();
    let mut queues_guard = queues.write();

    let queue = queues_guard
        .entry(queue_key)
        .or_insert_with(|| crossbeam::queue::ArrayQueue::new(RECV_QUEUE_LIMIT));
    // 瀛樺�?(source_window_id, payload)锛宻ource_window_id 灏辨槸鍙戦€佹秷鎭殑绐楀�?
    let _ = queue.push((window_id, data));

    // 鏇存柊鏈€杩戞秷鎭潵婧愮獥鍙ｏ紙鐢ㄤ簬 UNITYCAST 瀹氬悜鍥炲�?
    let last_source = get_last_source_windows();
    last_source.write().insert(handle_id.to_string(), window_id);
}

// === Stream 璁㈤槄绠＄悊锛坔andle_id -> 璁㈤槄淇℃伅�?===
type StreamSubscriptions = HashMap<String, Vec<u64>>;
static STREAM_SUBSCRIPTIONS: OnceLock<Arc<parking_lot::RwLock<StreamSubscriptions>>> =
    OnceLock::new();

#[inline]
fn get_stream_subscriptions_storage() -> &'static Arc<parking_lot::RwLock<StreamSubscriptions>> {
    STREAM_SUBSCRIPTIONS
        .get_or_init(|| Arc::new(parking_lot::RwLock::new(HashMap::with_capacity(16))))
}

/// 娉ㄥ�?stream 璁㈤槄锛坔andle_id �?window_id 鐨勭粦瀹氬叧绯伙紝甯﹀幓閲嶏級
fn register_stream_subscription(handle_id: String, window_id: u64) {
    let subscriptions = get_stream_subscriptions_storage();
    let mut subs = subscriptions.write();
    subs.entry(handle_id.clone())
        .or_insert_with(Vec::new);
    // 鍘婚噸锛氬鏋滃凡璁㈤槄鍒欎笉鍐嶆坊�?
    let ids = subs.get_mut(&handle_id).unwrap(); // safe: just inserted above if missing
    if !ids.contains(&window_id) {
        ids.push(window_id);
    }
}

/// 鑾峰彇鎸囧畾 handle_id 鐨勬墍鏈夎闃呯獥鍙?
pub(crate) fn get_stream_subscriptions(handle_id: &str) -> Vec<u64> {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id).cloned().unwrap_or_default()
}

// === Stream 鍙戦€佹暟鎹埌鍓嶇锛堝紓姝ョ増鏈紝鏀寔 Python await�?===
#[pyfunction(name = "rust_stream_send")]
#[pyo3(signature = (payload, handle_id, send_mode, window_ids=None, save_history=None))]
pub fn stream_send<'py>(
    py: Python<'py>,
    payload: Bound<'_, PyAny>,
    handle_id: String,
    send_mode: String,
    window_ids: Option<Vec<u64>>,
    save_history: Option<bool>,
) -> PyResult<Bound<'py, PyAny>> {
    let t_entry = std::time::Instant::now();

    // �?Python payload 杞崲涓?JSON
    let payload_json: Value =
        Python::attach(|_py| pythonize::depythonize(&payload).unwrap_or(Value::Null));

    // 鏋勫缓鍝嶅簲娑堟�?
    let response = serde_json::json!({
        "handle_id": handle_id,
        "handle_type": "stream",
        "request_id": null,
        "payload": payload_json
    });

    // 存储到历史消息（默认保存，save_history=False 时跳过）
    if save_history.unwrap_or(true) {
        store_stream_history(&handle_id, payload_json);
    }

    // 鏋勫�?JavaScript 浠ｇ爜锛堜娇�?Arc 閬垮厤澶氱獥鍙ｅ箍鎾椂鐨勫瓧绗︿覆 clone�?
    let js_code = Arc::new(format!(
        "window.__pywebron_dispatch({})",
        serde_json::to_string(&response).unwrap_or_default()
    ));
    let _t_build_done = t_entry.elapsed();

    // 鏍规嵁鍙戦€佹ā寮忓拰 window_ids 纭畾鐩爣绐楀�?
    let target_windows: Vec<u64> = match send_mode.as_str() {
        "broadcast" => crate::app::get_all_window_ids(),
        "multicast" => window_ids.unwrap_or_default(),
        "unitycast" => {
            // �?LAST_SOURCE_WINDOWS 鑾峰彇鏈€杩戠殑娑堟伅鏉ユ簮绐楀�?
            let last_source = LAST_SOURCE_WINDOWS.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))));
            let source_map = last_source.read();
            source_map.get(&handle_id).map(|&id| vec![id]).unwrap_or_default()
        }
        _ => {
            let subscribed = get_stream_subscriptions(&handle_id);
            if !subscribed.is_empty() {
                subscribed
            } else {
                crate::app::get_all_window_ids()
            }
        }
    };

    // 鍙戦€佹秷鎭埌鐩爣绐楀彛锛圓rc 鍏变韩寮曠敤锛岄伩鍏嶉噸�?clone 瀛楃涓诧級
    for window_id in target_windows {
        crate::app::send_script_to_window(window_id, std::sync::Arc::clone(&js_code));
    }

    // 杩斿洖涓€涓彲 await �?Future锛堝疄闄呭凡鍚屾瀹屾垚锛屼絾鏀寔 Python await 璇硶锛?
    pyo3_async_runtimes::tokio::future_into_py(py, async move { Ok(()) })
}


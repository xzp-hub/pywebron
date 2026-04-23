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

type StreamMessage = (u64, Value);
type StreamInbox = (
    crossbeam::channel::Sender<StreamMessage>,
    crossbeam::channel::Receiver<StreamMessage>,
);
type StreamInboxes = HashMap<String, StreamInbox>;

// Active stream handlers by handle_id.
static ACTIVE_HANDLERS: OnceLock<Arc<RwLock<HashSet<String>>>> = OnceLock::new();

// The most recent message source window for each stream handle.
type LastSourceMap = HashMap<String, u64>;
static LAST_SOURCE_WINDOWS: OnceLock<Arc<RwLock<LastSourceMap>>> = OnceLock::new();

// Saved stream history by handle_id.
type StreamHistory = HashMap<String, Vec<Value>>;
static STREAM_HISTORY: OnceLock<Arc<RwLock<StreamHistory>>> = OnceLock::new();
const HISTORY_LIMIT: usize = 200;

// Per-handle stream inbox, consumed by the single active Python stream handler.
static STREAM_INBOXES: OnceLock<Arc<RwLock<StreamInboxes>>> = OnceLock::new();
const RECV_QUEUE_LIMIT: usize = 100;

#[inline]
pub(crate) fn get_active_handlers() -> &'static Arc<RwLock<HashSet<String>>> {
    ACTIVE_HANDLERS.get_or_init(|| Arc::new(RwLock::new(HashSet::with_capacity(16))))
}

pub fn is_handler_active(handle_id: &str) -> bool {
    let active = get_active_handlers();
    active.read().contains(handle_id)
}

#[inline]
fn get_stream_history() -> &'static Arc<RwLock<StreamHistory>> {
    STREAM_HISTORY.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

#[inline]
fn get_last_source_windows() -> &'static Arc<RwLock<LastSourceMap>> {
    LAST_SOURCE_WINDOWS.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

#[inline]
fn get_stream_inboxes() -> &'static Arc<RwLock<StreamInboxes>> {
    STREAM_INBOXES.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

fn ensure_stream_sender(handle_id: &str) -> crossbeam::channel::Sender<StreamMessage> {
    let inboxes = get_stream_inboxes();
    let mut guard = inboxes.write();
    let entry = guard
        .entry(handle_id.to_string())
        .or_insert_with(|| crossbeam::channel::bounded(RECV_QUEUE_LIMIT));
    entry.0.clone()
}

fn ensure_stream_receiver(handle_id: &str) -> crossbeam::channel::Receiver<StreamMessage> {
    let inboxes = get_stream_inboxes();
    let mut guard = inboxes.write();
    let entry = guard
        .entry(handle_id.to_string())
        .or_insert_with(|| crossbeam::channel::bounded(RECV_QUEUE_LIMIT));
    entry.1.clone()
}

pub(crate) fn store_stream_history(handle_id: &str, payload: Value) {
    let history = get_stream_history();
    let mut h = history.write();
    let msgs = h.entry(handle_id.to_string()).or_insert_with(Vec::new);
    if msgs.len() >= HISTORY_LIMIT {
        msgs.remove(0);
    }
    msgs.push(payload);
}

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

#[pyfunction(name = "rust_stream_recv")]
pub fn stream_recv<'py>(py: Python<'py>, handle_id: String) -> PyResult<Bound<'py, PyAny>> {
    let hid = handle_id.clone();
    let receiver = ensure_stream_receiver(&handle_id);
    let future = pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let result = tokio::task::spawn_blocking(move || receiver.recv().ok())
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        if let Some((source_window_id, payload)) = result {
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
        }
    })?;

    Ok(future)
}

pub(crate) fn is_stream_active(handle_id: &str, window_id: u64) -> bool {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id)
        .map_or(false, |ids| ids.contains(&window_id))
}

pub(crate) fn register_stream_window(handle_id: &str, window_id: u64) {
    register_stream_subscription(handle_id.to_string(), window_id);
    let _ = ensure_stream_receiver(handle_id);
}

pub(crate) fn unregister_stream_window(handle_id: &str, window_id: u64) -> bool {
    let subscriptions = get_stream_subscriptions_storage();
    let mut subs = subscriptions.write();
    let mut removed = false;

    let should_remove_handle = if let Some(ids) = subs.get_mut(handle_id) {
        let before = ids.len();
        ids.retain(|id| *id != window_id);
        removed = ids.len() != before;
        ids.is_empty()
    } else {
        false
    };

    if should_remove_handle {
        subs.remove(handle_id);
    }

    let last_source = get_last_source_windows();
    let mut last_source_guard = last_source.write();
    if last_source_guard.get(handle_id) == Some(&window_id) {
        last_source_guard.remove(handle_id);
    }

    removed
}

pub(crate) fn cleanup_window_streams(window_id: u64) {
    let handle_ids: Vec<String> = {
        let subscriptions = get_stream_subscriptions_storage();
        let subs = subscriptions.read();
        subs.iter()
            .filter_map(|(handle_id, ids)| ids.contains(&window_id).then(|| handle_id.clone()))
            .collect()
    };

    for handle_id in handle_ids {
        unregister_stream_window(&handle_id, window_id);
    }
}

pub(crate) fn push_stream_data(handle_id: &str, window_id: u64, data: Value) {
    let sender = ensure_stream_sender(handle_id);
    let _ = sender.try_send((window_id, data));

    let last_source = get_last_source_windows();
    last_source.write().insert(handle_id.to_string(), window_id);
}

type StreamSubscriptions = HashMap<String, Vec<u64>>;
static STREAM_SUBSCRIPTIONS: OnceLock<Arc<parking_lot::RwLock<StreamSubscriptions>>> =
    OnceLock::new();

#[inline]
fn get_stream_subscriptions_storage() -> &'static Arc<parking_lot::RwLock<StreamSubscriptions>> {
    STREAM_SUBSCRIPTIONS
        .get_or_init(|| Arc::new(parking_lot::RwLock::new(HashMap::with_capacity(16))))
}

fn register_stream_subscription(handle_id: String, window_id: u64) {
    let subscriptions = get_stream_subscriptions_storage();
    let mut subs = subscriptions.write();
    let ids = subs.entry(handle_id).or_insert_with(Vec::new);
    if !ids.contains(&window_id) {
        ids.push(window_id);
    }
}

pub(crate) fn get_stream_subscriptions(handle_id: &str) -> Vec<u64> {
    let subscriptions = get_stream_subscriptions_storage();
    let subs = subscriptions.read();
    subs.get(handle_id).cloned().unwrap_or_default()
}

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
    let payload_json: Value =
        Python::attach(|_py| pythonize::depythonize(&payload).unwrap_or(Value::Null));

    let response = serde_json::json!({
        "handle_id": handle_id,
        "handle_type": "stream",
        "request_id": null,
        "payload": payload_json
    });

    if save_history.unwrap_or(true) {
        store_stream_history(&handle_id, payload_json);
    }

    let js_code = Arc::new(format!(
        "window.__pywebron_dispatch({})",
        serde_json::to_string(&response).unwrap_or_default()
    ));

    let target_windows: Vec<u64> = match send_mode.as_str() {
        "broadcast" => crate::app::get_all_window_ids(),
        "multicast" => window_ids.unwrap_or_default(),
        "unitycast" => {
            let last_source = get_last_source_windows();
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

    for window_id in target_windows {
        crate::app::send_script_to_window(window_id, std::sync::Arc::clone(&js_code));
    }

    pyo3_async_runtimes::tokio::future_into_py(py, async move { Ok(()) })
}

use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::OnceLock;

pub fn load_js_api() -> &'static str {
    static JS_API: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(|| include_str!("../../builtins/pywebron.js").to_string());
    &JS_API
}

type StreamMessage = (u64, Value);
type StreamInbox = (
    crossbeam::channel::Sender<StreamMessage>,
    crossbeam::channel::Receiver<StreamMessage>,
);
type StreamInboxes = HashMap<String, StreamInbox>;
type StreamHistory = HashMap<String, VecDeque<Value>>;
type LastSourceMap = HashMap<String, u64>;
type StreamSubscriptions = HashMap<String, HashSet<u64>>;
type WindowStreams = HashMap<u64, HashSet<String>>;

static ACTIVE_HANDLERS: OnceLock<Arc<RwLock<HashSet<String>>>> = OnceLock::new();
static LAST_SOURCE_WINDOWS: OnceLock<Arc<RwLock<LastSourceMap>>> = OnceLock::new();
static STREAM_HISTORY: OnceLock<Arc<RwLock<StreamHistory>>> = OnceLock::new();
static STREAM_INBOXES: OnceLock<Arc<RwLock<StreamInboxes>>> = OnceLock::new();
static STREAM_SUBSCRIPTIONS: OnceLock<Arc<RwLock<StreamSubscriptions>>> = OnceLock::new();
static WINDOW_STREAMS: OnceLock<Arc<RwLock<WindowStreams>>> = OnceLock::new();

const HISTORY_LIMIT: usize = 200;
const RECV_QUEUE_LIMIT: usize = 100;

#[inline]
pub(crate) fn get_active_handlers() -> &'static Arc<RwLock<HashSet<String>>> {
    ACTIVE_HANDLERS.get_or_init(|| Arc::new(RwLock::new(HashSet::with_capacity(16))))
}

pub fn is_handler_active(handle_id: &str) -> bool {
    get_active_handlers().read().contains(handle_id)
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

#[inline]
fn get_stream_subscriptions_storage() -> &'static Arc<RwLock<StreamSubscriptions>> {
    STREAM_SUBSCRIPTIONS.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

#[inline]
fn get_window_streams_storage() -> &'static Arc<RwLock<WindowStreams>> {
    WINDOW_STREAMS.get_or_init(|| Arc::new(RwLock::new(HashMap::with_capacity(16))))
}

fn ensure_stream_inbox(handle_id: &str) -> StreamInbox {
    let inboxes = get_stream_inboxes();
    let mut guard = inboxes.write();
    guard
        .entry(handle_id.to_string())
        .or_insert_with(|| crossbeam::channel::bounded(RECV_QUEUE_LIMIT))
        .clone()
}

fn ensure_stream_sender(handle_id: &str) -> crossbeam::channel::Sender<StreamMessage> {
    ensure_stream_inbox(handle_id).0
}

fn ensure_stream_receiver(handle_id: &str) -> crossbeam::channel::Receiver<StreamMessage> {
    ensure_stream_inbox(handle_id).1
}

fn cleanup_handle_state_if_idle(handle_id: &str) {
    let has_subscribers = {
        let subscriptions = get_stream_subscriptions_storage();
        subscriptions
            .read()
            .get(handle_id)
            .map(|ids| !ids.is_empty())
            .unwrap_or(false)
    };

    if has_subscribers || is_handler_active(handle_id) {
        return;
    }

    get_last_source_windows().write().remove(handle_id);
    get_stream_history().write().remove(handle_id);
    get_stream_inboxes().write().remove(handle_id);
}

pub(crate) fn store_stream_history(handle_id: &str, payload: Value) {
    let history = get_stream_history();
    let mut history = history.write();
    let messages = history
        .entry(handle_id.to_string())
        .or_insert_with(|| VecDeque::with_capacity(HISTORY_LIMIT));
    if messages.len() >= HISTORY_LIMIT {
        messages.pop_front();
    }
    messages.push_back(payload);
}

pub(crate) fn send_history_to_window(handle_id: &str, window_id: u64) {
    let messages = {
        let history = get_stream_history();
        let history = history.read();
        history
            .get(handle_id)
            .map(|items| items.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    };

    if messages.is_empty() {
        return;
    }

    let response = serde_json::json!({
        "handle_id": handle_id,
        "handle_type": "stream",
        "request_id": null,
        "payload": {
            "__history_batch__": true,
            "messages": messages
        }
    });
    let js_code = format!(
        "window.__pywebron_dispatch({})",
        serde_json::to_string(&response).unwrap_or_default()
    );
    crate::app::send_script_to_window(window_id, std::sync::Arc::new(js_code));
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
    let subscriptions = subscriptions.read();
    subscriptions
        .get(handle_id)
        .map_or(false, |ids| ids.contains(&window_id))
}

pub(crate) fn register_stream_window(handle_id: &str, window_id: u64) {
    {
        let subscriptions = get_stream_subscriptions_storage();
        let mut subscriptions = subscriptions.write();
        subscriptions
            .entry(handle_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(window_id);
    }

    {
        let window_streams = get_window_streams_storage();
        let mut window_streams = window_streams.write();
        window_streams
            .entry(window_id)
            .or_insert_with(HashSet::new)
            .insert(handle_id.to_string());
    }

    let _ = ensure_stream_receiver(handle_id);
}

pub(crate) fn unregister_stream_window(handle_id: &str, window_id: u64) -> bool {
    let removed = {
        let subscriptions = get_stream_subscriptions_storage();
        let mut subscriptions = subscriptions.write();
        if let Some(ids) = subscriptions.get_mut(handle_id) {
            let removed = ids.remove(&window_id);
            if ids.is_empty() {
                subscriptions.remove(handle_id);
            }
            removed
        } else {
            false
        }
    };

    {
        let window_streams = get_window_streams_storage();
        let mut window_streams = window_streams.write();
        if let Some(handles) = window_streams.get_mut(&window_id) {
            handles.remove(handle_id);
            if handles.is_empty() {
                window_streams.remove(&window_id);
            }
        }
    }

    {
        let last_source = get_last_source_windows();
        let mut last_source = last_source.write();
        if last_source.get(handle_id) == Some(&window_id) {
            last_source.remove(handle_id);
        }
    }

    cleanup_handle_state_if_idle(handle_id);
    removed
}

pub(crate) fn cleanup_window_streams(window_id: u64) {
    let handle_ids = {
        let window_streams = get_window_streams_storage();
        let mut window_streams = window_streams.write();
        window_streams
            .remove(&window_id)
            .map(|handles| handles.into_iter().collect::<Vec<_>>())
            .unwrap_or_default()
    };

    for handle_id in handle_ids {
        unregister_stream_window(&handle_id, window_id);
    }
}

pub(crate) fn push_stream_data(handle_id: &str, window_id: u64, data: Value) {
    let sender = ensure_stream_sender(handle_id);
    match sender.try_send((window_id, data)) {
        Ok(()) => {}
        Err(crossbeam::channel::TrySendError::Full((window_id, data))) => {
            let receiver = ensure_stream_receiver(handle_id);
            let _ = receiver.try_recv();
            let _ = sender.try_send((window_id, data));
        }
        Err(crossbeam::channel::TrySendError::Disconnected(_)) => {}
    }

    get_last_source_windows()
        .write()
        .insert(handle_id.to_string(), window_id);
}

pub(crate) fn get_stream_subscriptions(handle_id: &str) -> Vec<u64> {
    let subscriptions = get_stream_subscriptions_storage();
    let subscriptions = subscriptions.read();
    subscriptions
        .get(handle_id)
        .map(|ids| ids.iter().copied().collect())
        .unwrap_or_default()
}

#[pyfunction(name = "rust_stream_send")]
#[pyo3(signature = (payload, handle_id, send_mode, window_ids=None, save_history=None))]
pub fn stream_send(
    payload: Bound<'_, PyAny>,
    handle_id: String,
    send_mode: String,
    window_ids: Option<Vec<u64>>,
    save_history: Option<bool>,
) -> PyResult<bool> {
    let payload_json: Value = pythonize::depythonize(&payload).unwrap_or(Value::Null);

    let response = serde_json::json!({
        "handle_id": handle_id,
        "handle_type": "stream",
        "request_id": null,
        "payload": payload_json
    });

    if save_history.unwrap_or(true) {
        if let Some(payload) = response.get("payload").cloned() {
            store_stream_history(&handle_id, payload);
        }
    }

    let js_code = Arc::new(format!(
        "window.__pywebron_dispatch({})",
        serde_json::to_string(&response).unwrap_or_default()
    ));

    let target_windows: Vec<u64> = match send_mode.as_str() {
        "broadcast" => crate::app::get_all_window_ids(),
        "multicast" => window_ids.unwrap_or_default(),
        "unitycast" => get_last_source_windows()
            .read()
            .get(&handle_id)
            .map(|&id| vec![id])
            .unwrap_or_default(),
        _ => {
            let subscribed = get_stream_subscriptions(&handle_id);
            if subscribed.is_empty() {
                crate::app::get_all_window_ids()
            } else {
                subscribed
            }
        }
    };

    for window_id in target_windows {
        crate::app::send_script_to_window(window_id, Arc::clone(&js_code));
    }

    Ok(true)
}

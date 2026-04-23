use crossbeam::channel;
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use pyo3_async_runtimes::TaskLocals;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::{LazyLock, OnceLock};
use tao::event_loop::EventLoopProxy;
use tokio::sync::oneshot;

use crate::configs::UserEvent;

type HandleCache = std::collections::HashMap<String, Py<PyAny>>;
static HANDLE_CACHE: LazyLock<parking_lot::RwLock<HandleCache>> =
    LazyLock::new(|| parking_lot::RwLock::new(HandleCache::with_capacity(32)));

type HandlerFuture = Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>;

static PYTHON_HANDLER_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    let worker_threads = std::env::var("PYWEBRON_PY_ASYNC_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(1, 4))
        .unwrap_or(2);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .thread_name("pywebron-pyasync")
        .build()
        .expect("failed to create pywebron async runtime")
});

static PYTHON_TASK_LOCALS: LazyLock<Result<TaskLocals, String>> =
    LazyLock::new(init_python_task_locals);

static LOG_DEBUG: LazyLock<bool> = LazyLock::new(|| {
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
fn debug_log(message: impl FnOnce() -> String) {
    if *LOG_DEBUG {
        eprintln!("{}", message());
    }
}

#[inline]
fn cache_key(handle_id: &str, handler_type: &str) -> String {
    format!("{}:{}", handler_type, handle_id)
}

#[inline]
fn queue_capacity(env_key: &str, default: usize) -> usize {
    std::env::var(env_key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.max(1))
        .unwrap_or(default)
}

#[inline]
fn thread_count(env_key: &str, default: usize, min: usize, max: usize) -> usize {
    std::env::var(env_key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(min, max))
        .unwrap_or(default.clamp(min, max))
}

fn find_handler<'py>(
    py: Python<'py>,
    handle_id: &str,
    handler_type: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let configs = py.import("pywebron.configs")?;
    let handle_index = configs.getattr("HANDLE_INDEX")?;
    let mapping = handle_index.call_method1("__getitem__", (handler_type,))?;
    let mapping = mapping.cast::<PyDict>()?;
    if mapping.contains(handle_id)? {
        return Ok(Some(mapping.call_method1("__getitem__", (handle_id,))?));
    }
    Ok(None)
}

fn get_handler<'py>(
    py: Python<'py>,
    handle_id: &str,
    handler_type: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let handler_cache_key = cache_key(handle_id, handler_type);
    let cache = HANDLE_CACHE.read();
    if let Some(handler) = cache.get(&handler_cache_key) {
        return Ok(handler.bind(py).to_owned());
    }
    drop(cache);

    if let Some(handler) = find_handler(py, handle_id, handler_type)? {
        HANDLE_CACHE
            .write()
            .insert(handler_cache_key, handler.clone().unbind());
        Ok(handler)
    } else {
        Err(pyo3::exceptions::PyValueError::new_err(format!(
            "{} handler not found: {}",
            if handler_type == "invoke" {
                "Invoke"
            } else {
                "Stream"
            },
            handle_id
        )))
    }
}

fn init_python_task_locals() -> Result<TaskLocals, String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::Builder::new()
        .name("pywebron-python-loop".to_string())
        .spawn(move || {
            let init_result = Python::attach(|py| -> PyResult<TaskLocals> {
                let asyncio = py.import("asyncio")?;
                let event_loop = asyncio.call_method0("new_event_loop")?;
                asyncio.call_method1("set_event_loop", (event_loop.clone(),))?;
                TaskLocals::new(event_loop).copy_context(py)
            })
            .map_err(|e| e.to_string());

            match init_result {
                Ok(locals) => {
                    let _ = tx.send(Ok(locals.clone()));
                    let run_result = Python::attach(|py| -> PyResult<()> {
                        let asyncio = py.import("asyncio")?;
                        let event_loop = locals.event_loop(py);
                        asyncio.call_method1("set_event_loop", (event_loop.clone(),))?;
                        event_loop.call_method0("run_forever")?;
                        Ok(())
                    });
                    if let Err(err) = run_result {
                        eprintln!("[PyAsync] Python event loop stopped unexpectedly: {}", err);
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                }
            }
        })
        .map_err(|e| e.to_string())?;

    rx.recv()
        .map_err(|e| format!("failed to receive Python event loop locals: {}", e))?
}

fn python_task_locals() -> Result<TaskLocals, String> {
    match &*PYTHON_TASK_LOCALS {
        Ok(locals) => Ok(locals.clone()),
        Err(err) => Err(err.clone()),
    }
}

pub fn warm_python_runtime() -> PyResult<()> {
    python_task_locals()
        .map(|_| ())
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
}

fn make_handler_future(
    handle_id: String,
    handler_type: &'static str,
    window_id: u64,
    request_id: Option<String>,
    payload: Value,
) -> Result<HandlerFuture, String> {
    let locals = python_task_locals()?;
    let future = Python::attach(|py| -> PyResult<_> {
        let handler = get_handler(py, &handle_id, handler_type)?;
        let py_request = PyDict::new(py);
        py_request.set_item("window_id", window_id)?;
        py_request.set_item("handle_id", &handle_id)?;
        py_request.set_item("request_id", &request_id)?;
        py_request.set_item("payload", pythonize::pythonize(py, &payload)?)?;
        let coroutine = handler.call1((py_request,))?;
        pyo3_async_runtimes::into_future_with_locals(&locals, coroutine)
    })
    .map_err(|e| e.to_string())?;

    Ok(Box::pin(async move {
        let py_result = future.await.map_err(|e| e.to_string())?;
        Python::attach(|py| {
            let bound = py_result.into_bound(py);
            pythonize::depythonize(&bound).map_err(|e| e.to_string())
        })
    }))
}

pub struct IpcRequest {
    pub handle_id: String,
    pub window_id: u64,
    pub request_id: Option<String>,
    pub payload: Value,
    pub proxy: Option<EventLoopProxy<UserEvent>>,
    pub result_tx: Option<oneshot::Sender<Result<Value, String>>>,
}

static INVOKE_TX: OnceLock<channel::Sender<IpcRequest>> = OnceLock::new();
static STREAM_TX: OnceLock<channel::Sender<IpcRequest>> = OnceLock::new();

fn ensure_invoke_pool() -> &'static channel::Sender<IpcRequest> {
    INVOKE_TX.get_or_init(|| {
        let capacity = queue_capacity("PYWEBRON_INVOKE_QUEUE_CAPACITY", 256);
        let (tx, rx) = channel::bounded::<IpcRequest>(capacity);
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let num_threads = thread_count("PYWEBRON_INVOKE_THREADS", parallelism.clamp(2, 4), 1, 8);

        for index in 0..num_threads {
            let rx = rx.clone();
            std::thread::Builder::new()
                .name(format!("pywebron-invoke-{}", index))
                .spawn(move || {
                    while let Ok(request) = rx.recv() {
                        process_invoke_request(request);
                    }
                })
                .expect("Failed to spawn invoke worker");
        }

        tx
    })
}

fn ensure_stream_pool() -> &'static channel::Sender<IpcRequest> {
    STREAM_TX.get_or_init(|| {
        let capacity = queue_capacity("PYWEBRON_STREAM_QUEUE_CAPACITY", 128);
        let (tx, rx) = channel::bounded::<IpcRequest>(capacity);
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        let num_threads = thread_count("PYWEBRON_STREAM_THREADS", parallelism.clamp(1, 2), 1, 4);

        for index in 0..num_threads {
            let rx = rx.clone();
            std::thread::Builder::new()
                .name(format!("pywebron-stream-{}", index))
                .spawn(move || {
                    while let Ok(request) = rx.recv() {
                        process_stream_request(request);
                    }
                })
                .expect("Failed to spawn stream worker");
        }

        tx
    })
}

fn process_invoke_request(request: IpcRequest) {
    let IpcRequest {
        handle_id,
        window_id,
        request_id,
        payload,
        proxy,
        result_tx,
    } = request;

    let total_start = std::time::Instant::now();
    debug_log(|| {
        format!(
            "[Invoke] start handle={} wid={} req_id={:?}",
            handle_id, window_id, request_id
        )
    });

    let result = match make_handler_future(
        handle_id.clone(),
        "invoke",
        window_id,
        request_id.clone(),
        payload,
    ) {
        Ok(future) => {
            let (tx, rx) = oneshot::channel::<Result<Value, String>>();
            PYTHON_HANDLER_RUNTIME.spawn(async move {
                let _ = tx.send(future.await);
            });
            rx.blocking_recv()
                .unwrap_or_else(|_| Err("Invoke async worker channel closed".to_string()))
        }
        Err(err) => Err(err),
    };

    debug_log(|| {
        format!(
            "[Invoke] done handle={} elapsed={:?}",
            handle_id,
            total_start.elapsed()
        )
    });

    if let Some(tx) = result_tx {
        let _ = tx.send(result);
        return;
    }

    if let Some(proxy) = proxy {
        let response = match result {
            Ok(mut value) => {
                if let Some(object) = value.as_object_mut() {
                    object.insert("handle_type".into(), Value::String("invoke".to_string()));
                    object.insert(
                        "request_id".into(),
                        request_id
                            .as_ref()
                            .map(|item| Value::String(item.clone()))
                            .unwrap_or(Value::Null),
                    );
                }
                value
            }
            Err(error) => serde_json::json!({
                "window_id": window_id,
                "handle_id": handle_id,
                "handle_type": "invoke",
                "request_id": request_id,
                "payload": {
                    "code": 500,
                    "mssg": error,
                    "data": null
                }
            }),
        };

        let js_code = format!(
            "window.__pywebron_dispatch({})",
            serde_json::to_string(&response).unwrap_or_default()
        );
        let _ = proxy.send_event(UserEvent::EvaluateScript {
            window_id,
            script: std::sync::Arc::new(js_code),
        });
    }
}

fn process_stream_request(request: IpcRequest) {
    let IpcRequest {
        handle_id,
        window_id,
        request_id,
        payload,
        proxy: _,
        result_tx: _,
    } = request;

    debug_log(|| {
        format!(
            "[Stream] start handle={} wid={} req_id={:?}",
            handle_id, window_id, request_id
        )
    });

    crate::app::stream::register_stream_window(&handle_id, window_id);
    let active_handlers = crate::app::stream::get_active_handlers();
    active_handlers.write().insert(handle_id.clone());

    let future =
        match make_handler_future(handle_id.clone(), "stream", window_id, request_id, payload) {
            Ok(future) => future,
            Err(err) => {
                active_handlers.write().remove(&handle_id);
                eprintln!(
                    "[Stream] Handler start failed: {} | handle={}",
                    err, handle_id
                );
                return;
            }
        };

    PYTHON_HANDLER_RUNTIME.spawn(async move {
        let result = future.await;
        active_handlers.write().remove(&handle_id);
        if let Err(err) = result {
            eprintln!("[Stream] Handler error: {} | handle={}", err, handle_id);
        }
    });
}

pub fn shutdown_python_runtime() {
    if let Ok(locals) = python_task_locals() {
        Python::attach(|py| {
            let event_loop = locals.event_loop(py);
            let _ = event_loop.call_method0("stop");
        });
    }
}

pub fn submit_invoke_ipc(
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Value,
    proxy: EventLoopProxy<UserEvent>,
) -> Result<(), String> {
    ensure_invoke_pool()
        .try_send(IpcRequest {
            handle_id,
            window_id,
            request_id,
            payload,
            proxy: Some(proxy),
            result_tx: None,
        })
        .map_err(|err| match err {
            channel::TrySendError::Full(_) => "invoke queue is full".to_string(),
            channel::TrySendError::Disconnected(_) => "invoke queue is closed".to_string(),
        })
}

pub fn submit_stream_ipc(
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Value,
) -> Result<(), String> {
    ensure_stream_pool()
        .try_send(IpcRequest {
            handle_id,
            window_id,
            request_id,
            payload,
            proxy: None,
            result_tx: None,
        })
        .map_err(|err| match err {
            channel::TrySendError::Full(_) => "stream queue is full".to_string(),
            channel::TrySendError::Disconnected(_) => "stream queue is closed".to_string(),
        })
}

#[pyfunction(name = "rust_invoke_handle")]
pub fn invoke_handle<'py>(
    py: Python<'py>,
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let (tx, rx) = oneshot::channel::<Result<Value, String>>();
    let payload_value: Value = pythonize::depythonize(&payload).unwrap_or(Value::Null);

    ensure_invoke_pool()
        .try_send(IpcRequest {
            handle_id,
            window_id,
            request_id,
            payload: payload_value,
            proxy: None,
            result_tx: Some(tx),
        })
        .map_err(|err| {
            pyo3::exceptions::PyRuntimeError::new_err(match err {
                channel::TrySendError::Full(_) => "invoke queue is full".to_string(),
                channel::TrySendError::Disconnected(_) => "invoke queue is closed".to_string(),
            })
        })?;

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        match rx.await {
            Ok(Ok(value)) => Python::attach(|py| {
                let py_obj = pythonize::pythonize(py, &value)?;
                Ok(py_obj.unbind())
            }),
            Ok(Err(err)) => Err(pyo3::exceptions::PyRuntimeError::new_err(err).into()),
            Err(_) => {
                Err(pyo3::exceptions::PyRuntimeError::new_err("Invoke channel closed").into())
            }
        }
    })
}

#[pyfunction(name = "rust_get_handles")]
pub fn get_handles(py: Python<'_>) -> PyResult<Bound<'_, pyo3::types::PyDict>> {
    let result_dict = pyo3::types::PyDict::new(py);
    let invoke_dict = pyo3::types::PyDict::new(py);
    let stream_dict = pyo3::types::PyDict::new(py);

    let configs_module = py.import("pywebron.configs")?;
    let handle_index = configs_module.getattr("HANDLE_INDEX")?;

    let invoke_index = handle_index.call_method1("__getitem__", ("invoke",))?;
    let invoke_index = invoke_index.cast::<PyDict>()?;
    for item in invoke_index.items().try_iter()? {
        let item = item?;
        let (name, handler): (String, Bound<'_, PyAny>) = item.extract()?;
        let handler_name = handler.getattr("__name__")?.extract::<String>()?;
        invoke_dict.set_item(name, handler_name)?;
    }

    let stream_index = handle_index.call_method1("__getitem__", ("stream",))?;
    let stream_index = stream_index.cast::<PyDict>()?;
    for item in stream_index.items().try_iter()? {
        let item = item?;
        let (name, handler): (String, Bound<'_, PyAny>) = item.extract()?;
        let handler_name = handler.getattr("__name__")?.extract::<String>()?;
        stream_dict.set_item(name, handler_name)?;
    }

    result_dict.set_item("invoke", invoke_dict)?;
    result_dict.set_item("stream", stream_dict)?;
    Ok(result_dict)
}

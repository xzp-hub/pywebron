use crossbeam::channel;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use serde_json::Value;
use std::sync::{LazyLock, OnceLock};
use tao::event_loop::EventLoopProxy;

use crate::configs::UserEvent;

// === Handler 缓存 ===
type HandleCache = std::collections::HashMap<String, Py<PyAny>>;
static HANDLE_CACHE: LazyLock<parking_lot::RwLock<HandleCache>> =
    LazyLock::new(|| parking_lot::RwLock::new(HandleCache::with_capacity(32)));

#[inline]
fn cache_key(handle_id: &str, handler_type: &str) -> String {
    format!("{}:{}", handler_type, handle_id)
}

/// 从 HANDLES 字典中按 handle_id 和 handler_type 查找 handler
/// HANDLES 结构: { group_name: [ {"name": str, "type": "invoke"|"stream", "handler": callable}, ... ] }
fn find_handler<'py>(
    py: Python<'py>,
    handle_id: &str,
    handler_type: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let configs = py.import("pywebron.configs")?;
    let handles = configs.getattr("HANDLES")?;
    eprintln!("[DEBUG-RUST] 开始查找 handler: {} (type={})", handle_id, handler_type);
    // 使用 .items() 方法来遍历字典的键值对
    let items_method = handles.getattr("items")?;
    let items = items_method.call0()?;
    let groups = items.try_iter()?;
    for group_result in groups {
        let group = group_result?;
        eprintln!("[DEBUG-RUST] group type: {:?}", group.get_type().name());
        // group 是 (key, value) 元组
        let extracted: Result<(String, Bound<'_, PyAny>), _> = group.extract();
        match extracted {
            Ok((key, handler_list)) => {
                eprintln!("[DEBUG-RUST] 遍历组: {}", key);
                let items = handler_list.try_iter()?;
                for item_result in items {
                    let item = item_result?;
                    let name: String = item.get_item("name")?.extract()?;
                    let htype: String = item.get_item("type")?.extract()?;
                    if name == handle_id && htype == handler_type {
                        let handler = item.get_item("handler")?;
                        eprintln!("[DEBUG-RUST] 找到 handler: {}", handle_id);
                        return Ok(Some(handler));
                    }
                }
            }
            Err(e) => {
                eprintln!("[DEBUG-RUST] 提取失败: {}", e);
                return Err(e);
            }
        }
    }
    eprintln!("[DEBUG-RUST] 未找到 handler: {}", handle_id);
    Ok(None)
}

// === 统一的 IPC 处理线程池 ===
// 所有 IPC 请求（invoke + stream）都在这个线程池中处理

/// IPC 请求统一结构
pub struct IpcRequest {
    pub handle_id: String,
    pub window_id: u64,
    pub request_id: Option<String>,
    pub payload: Value,
    /// IPC 模式：通过事件循环将结果发送到前端
    pub proxy: Option<EventLoopProxy<UserEvent>>,
    /// Python 调用模式：通过 channel 将结果返回给调用方
    pub result_tx: Option<channel::Sender<Result<Value, String>>>,
    /// 是否为 stream handler
    pub is_stream: bool,
}

// Invoke 线程池（用于处理 invoke 请求）
static INVOKE_TX: OnceLock<channel::Sender<IpcRequest>> = OnceLock::new();

// Stream 线程池（用于处理 stream 请求，独立避免阻塞 invoke）
static STREAM_TX: OnceLock<channel::Sender<IpcRequest>> = OnceLock::new();

/// 确保 invoke 线程池已初始化
fn ensure_invoke_pool() -> &'static channel::Sender<IpcRequest> {
    INVOKE_TX.get_or_init(|| {
        let (tx, rx) = channel::unbounded::<IpcRequest>();
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get().max(2))
            .unwrap_or(4);


        for i in 0..num_threads {
            let rx = rx.clone();
            std::thread::Builder::new()
                .name(format!("pywebron-invoke-{}", i))
                .spawn(move || loop {
                    match rx.recv() {
                        Ok(req) => process_invoke_request(req),
                        Err(_) => break,
                    }
                })
                .expect("Failed to spawn invoke worker");
        }

        tx
    })
}

/// 确保 stream 线程池已初始化
fn ensure_stream_pool() -> &'static channel::Sender<IpcRequest> {
    STREAM_TX.get_or_init(|| {
        let (tx, rx) = channel::unbounded::<IpcRequest>();
        // Stream 线程池不需要太多线程，因为每个 stream handler 是长期运行的
        let num_threads = 8;

        for i in 0..num_threads {
            let rx = rx.clone();
            std::thread::Builder::new()
                .name(format!("pywebron-stream-{}", i))
                .spawn(move || loop {
                    match rx.recv() {
                        Ok(req) => process_stream_request(req),
                        Err(_) => break,
                    }
                })
                .expect("Failed to spawn stream worker");
        }

        tx
    })
}

/// 处理 invoke 请求
fn process_invoke_request(request: IpcRequest) {
    let handle_id = request.handle_id.clone();
    let window_id = request.window_id;
    let request_id = request.request_id.clone();
    let payload = request.payload.clone();

    let t_total = std::time::Instant::now();
    eprintln!(
        "[Invoke] >>> 开始调用 | handle={} | wid={} | req_id={:?}",
        handle_id, window_id, request_id
    );

    // 获取 Python GIL，执行 handler
    let mut result: Result<Value, String> = Python::attach(|py| -> PyResult<Value> {
        let handler_cache_key = cache_key(&handle_id, "invoke");
        // 优先从缓存获取 handler，避免重复 Python import
        let handler = {
            let cache = HANDLE_CACHE.read();
            if let Some(h) = cache.get(&handler_cache_key) {
                h.bind(py).to_owned()
            } else {
                drop(cache);
                if let Some(h) = find_handler(py, &handle_id, "invoke")? {
                    HANDLE_CACHE
                        .write()
                        .insert(handler_cache_key.clone(), h.clone().unbind());
                    h
                } else {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Invoke handler not found: {}",
                        handle_id
                    )));
                }
            }
        };

        // 构建请求对象
        let request_obj = serde_json::json!({
            "window_id": window_id,
            "handle_id": &handle_id,
            "request_id": &request_id,
            "payload": payload
        });
        let py_request = pythonize::pythonize(py, &request_obj)?;
        let coroutine = handler.call1((py_request,))?;
        // eprintln!("[Invoke]     协程已创建 | handle={}", handle_id);

        // 使用 asyncio.run() 运行协程
        let asyncio = py.import("asyncio")?;
        eprintln!("[Invoke]     调用 asyncio.run() | handle={}", handle_id);
        let py_result = asyncio.call_method1("run", (coroutine,))?;
        eprintln!("[Invoke]     协程执行完毕 | handle={}", handle_id);

        // 解析结果
        let value: Value = pythonize::depythonize(&py_result)?;
        Ok(value)
    })
    .map_err(|e| e.to_string());

    let _elapsed = t_total.elapsed();
    eprintln!("[Invoke] 全流程耗时：{:?} | handle={}", _elapsed, handle_id);

    // 根据模式处理结果
    if let Some(tx) = request.result_tx {
        let _ = tx.send(result);
    } else if let Some(proxy) = request.proxy {
        let response = match result {
            Ok(ref mut r) => {
                if let Some(obj) = r.as_object_mut() {
                    obj.insert("handle_type".into(), serde_json::json!("invoke"));
                    obj.insert("request_id".into(), serde_json::json!(request_id));
                }
                r.clone()
            }
            Err(ref e) => serde_json::json!({
                "window_id": window_id,
                "handle_id": handle_id,
                "handle_type": "invoke",
                "request_id": request_id,
                "payload": {
                    "code": 500,
                    "mssg": e,
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
    // eprintln!("[Invoke] <<< 调用完成 | handle={}", handle_id);
}

/// 处理 stream 请求
fn process_stream_request(request: IpcRequest) {
    let handle_id = request.handle_id.clone();
    let window_id = request.window_id;
    let request_id = request.request_id.clone();
    let payload = request.payload.clone();

    eprintln!(
        "[Stream] >>> 开始调用 | handle={} | wid={} | req_id={:?}",
        handle_id, window_id, request_id
    );

    // 注册订阅关系
    crate::app::stream::register_stream_window(&handle_id, window_id);

    // 标记 handler 为活跃状态
    let active_handlers = crate::app::stream::get_active_handlers();
    active_handlers.write().insert(handle_id.clone());

    // 获取 Python GIL，执行 handler
    let result: Result<Value, String> = Python::attach(|py| -> PyResult<Value> {
        let handler_cache_key = cache_key(&handle_id, "stream");
        // 优先从缓存获取 handler，避免重复 Python import
        let handler = {
            let cache = HANDLE_CACHE.read();
            if let Some(h) = cache.get(&handler_cache_key) {
                h.bind(py).to_owned()
            } else {
                drop(cache);
                if let Some(h) = find_handler(py, &handle_id, "stream")? {
                    HANDLE_CACHE
                        .write()
                        .insert(handler_cache_key.clone(), h.clone().unbind());
                    h
                } else {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Stream handler not found: {}",
                        handle_id
                    )));
                }
            }
        };

        // 构建请求对象
        let request_obj = serde_json::json!({
            "window_id": window_id,
            "handle_id": &handle_id,
            "request_id": &request_id,
            "payload": payload
        });
        let py_request = pythonize::pythonize(py, &request_obj)?;
        let coroutine = handler.call1((py_request,))?;
        // eprintln!("[Stream]     协程已创建 | handle={}", handle_id);

        // 使用 asyncio.run() 运行协程（stream handler 是无限循环）
        let asyncio = py.import("asyncio")?;
        // eprintln!("[Stream]     调用 asyncio.run() | handle={}", handle_id);
        let _ = asyncio.call_method1("run", (coroutine,))?;

        // Stream handler 通常不会返回（无限循环）
        // eprintln!("[Stream]     协程已结束（意外） | handle={}", handle_id);
        Ok(serde_json::json!({"started": true}))
    })
    .map_err(|e| e.to_string());

    // handler 结束后，移除活跃状态
    active_handlers.write().remove(&handle_id);

    if let Err(_e) = result {
        eprintln!("[Stream] Handler 错误：{} | handle={}", _e, handle_id);
    }
}

// === 公开 API ===

/// 提交 invoke 请求到线程池（IPC 模式：结果自动发回前端）
pub fn submit_invoke_ipc(
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Value,
    proxy: EventLoopProxy<UserEvent>,
) {
    let _ = ensure_invoke_pool().send(IpcRequest {
        handle_id,
        window_id,
        request_id,
        payload,
        proxy: Some(proxy),
        result_tx: None,
        is_stream: false,
    });
}

/// 提交 stream 请求到线程池
pub fn submit_stream_ipc(
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Value,
) {
    let _ = ensure_stream_pool().send(IpcRequest {
        handle_id,
        window_id,
        request_id,
        payload,
        proxy: None,
        result_tx: None,
        is_stream: true,
    });
}

/// Python 可调用：提交 invoke 到线程池，返回 future 供 Python await
#[pyfunction(name = "rust_invoke_handle")]
pub fn invoke_handle<'py>(
    py: Python<'py>,
    handle_id: String,
    window_id: u64,
    request_id: Option<String>,
    payload: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let (tx, rx) = channel::bounded::<Result<Value, String>>(1);
    let payload_value: Value = pythonize::depythonize(&payload).unwrap_or(Value::Null);

    let _ = ensure_invoke_pool().send(IpcRequest {
        handle_id: handle_id.clone(),
        window_id,
        request_id,
        payload: payload_value,
        proxy: None,
        result_tx: Some(tx),
        is_stream: false,
    });

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let result = tokio::task::spawn_blocking(move || {
            rx.recv()
                .unwrap_or(Err("Invoke channel closed".to_string()))
        })
        .await
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        match result {
            Ok(value) => Python::attach(|py| {
                let py_obj = pythonize::pythonize(py, &value)?;
                Ok(py_obj.unbind())
            }),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e).into()),
        }
    })
}

/// 获取所有处理器列表
#[pyfunction(name = "rust_get_handles")]
pub fn get_handles(py: Python<'_>) -> PyResult<Bound<'_, pyo3::types::PyDict>> {
    let result_dict = pyo3::types::PyDict::new(py);
    let invoke_dict = pyo3::types::PyDict::new(py);
    let stream_dict = pyo3::types::PyDict::new(py);

    let configs_module = py.import("pywebron.configs")?;
    let handles = configs_module.getattr("HANDLES")?;
    // 使用 .items() 方法来遍历字典的键值对
    let items_method = handles.getattr("items")?;
    let items = items_method.call0()?;
    let groups = items.try_iter()?;
    for group_result in groups {
        let group = group_result?;
        let (_, handler_list) = group.extract::<(String, Bound<'_, PyAny>)>()?;
        let items = handler_list.try_iter()?;
        for item_result in items {
            let item = item_result?;
            let name: String = item.get_item("name")?.extract()?;
            let htype: String = item.get_item("type")?.extract()?;
            let handler = item.get_item("handler")?;
            let handler_name = handler.getattr("__name__")?.extract::<String>()?;
            if htype == "invoke" {
                invoke_dict.set_item(name, handler_name)?;
            } else {
                stream_dict.set_item(name, handler_name)?;
            }
        }
    }

    result_dict.set_item("invoke", invoke_dict)?;
    result_dict.set_item("stream", stream_dict)?;

    Ok(result_dict)
}

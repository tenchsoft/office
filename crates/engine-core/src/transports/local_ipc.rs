#[cfg(unix)]
use std::io::{BufRead, BufReader, Write};

use serde_json::json;
use tench_shared_types::{ConnectionProfile, EngineEvent, EngineRequest, EngineResponse};

#[cfg(unix)]
use crate::wire::parse_sse_reader;
use crate::wire::transport_error_with_message;
use crate::{EngineClient, EngineEventStream, EngineTransport};

const DEFAULT_SOCK_PATH: &str = "/tmp/tench-engine.sock";

#[derive(Clone, Debug)]
pub struct LocalIpcTransport {
    endpoint: String,
}

impl LocalIpcTransport {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    fn socket_path(&self) -> &str {
        if self.endpoint.is_empty() {
            DEFAULT_SOCK_PATH
        } else {
            &self.endpoint
        }
    }
}

impl EngineClient for LocalIpcTransport {
    fn call(&self, request: EngineRequest) -> EngineResponse {
        ipc_call(self.socket_path(), request)
    }

    fn stream(&self, request: EngineRequest) -> EngineEventStream {
        ipc_stream(self.socket_path(), request)
    }

    fn cancel(&self, task_id: &str) -> EngineResponse {
        let request_id = format!("cancel_{task_id}");
        let payload = json!({ "task_id": task_id });
        ipc_call(
            self.socket_path(),
            EngineRequest::new(
                &request_id,
                tench_shared_types::EngineMethod::TasksCancel,
                payload,
            ),
        )
    }
}

impl EngineTransport for LocalIpcTransport {
    fn profile(&self) -> ConnectionProfile {
        ConnectionProfile::LocalIpc {
            endpoint: self.endpoint.clone(),
        }
    }
}

/// Send a JSON-RPC request over a Unix domain socket and read the response.
#[cfg(unix)]
fn ipc_call(socket_path: &str, request: EngineRequest) -> EngineResponse {
    let request_id = request.id.clone();

    let mut stream = match std::os::unix::net::UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(error) => {
            return EngineResponse::failure(
                &request_id,
                ipc_connect_error(&request_id, socket_path, &error),
            );
        }
    };

    let payload = match serde_json::to_vec(&request) {
        Ok(payload) => payload,
        Err(error) => {
            return EngineResponse::failure(&request_id, ipc_serialize_error(&request_id, &error));
        }
    };

    if let Err(error) = stream
        .write_all(&payload)
        .and_then(|_| stream.write_all(b"\n"))
    {
        return EngineResponse::failure(&request_id, ipc_write_error(&request_id, &error));
    }
    if let Err(error) = stream.flush() {
        return EngineResponse::failure(&request_id, ipc_write_error(&request_id, &error));
    }

    // Read the response as a JSON line
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => EngineResponse::failure(
            &request_id,
            transport_error_with_message(
                &request_id,
                "local_ipc",
                "IPC socket closed before response",
            ),
        ),
        Ok(_) => serde_json::from_str::<EngineResponse>(&line).unwrap_or_else(|error| {
            EngineResponse::failure(
                &request_id,
                transport_error_with_message(
                    &request_id,
                    "local_ipc",
                    format!("Failed to parse IPC response: {error}"),
                ),
            )
        }),
        Err(error) => EngineResponse::failure(
            &request_id,
            transport_error_with_message(
                &request_id,
                "local_ipc",
                format!("Failed to read from IPC socket: {error}"),
            ),
        ),
    }
}

/// Connect to the IPC socket and parse SSE events from the stream.
#[cfg(unix)]
fn ipc_stream(socket_path: &str, request: EngineRequest) -> EngineEventStream {
    let request_id = request.id.clone();

    // For non-streaming methods, fall back to call + response_to_events
    if request.method != tench_shared_types::EngineMethod::ChatCompletionsCreate {
        let response = ipc_call(socket_path, request);
        return crate::wire::response_to_events(response);
    }

    let mut stream = match std::os::unix::net::UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(error) => {
            return vec![EngineEvent::Error {
                error: ipc_connect_error(&request_id, socket_path, &error),
            }];
        }
    };

    // Build the streaming request payload
    let mut payload = request.params;
    if let Some(object) = payload.as_object_mut() {
        object.insert("id".to_string(), json!(request_id.clone()));
        object.insert("stream".to_string(), json!(true));
    }

    let payload_bytes = match serde_json::to_vec(&payload) {
        Ok(bytes) => bytes,
        Err(error) => {
            return vec![EngineEvent::Error {
                error: ipc_serialize_error(&request_id, &error),
            }];
        }
    };

    if let Err(error) = stream
        .write_all(&payload_bytes)
        .and_then(|_| stream.write_all(b"\n"))
    {
        return vec![EngineEvent::Error {
            error: ipc_write_error(&request_id, &error),
        }];
    }
    let _ = stream.flush();

    let reader = BufReader::new(stream);
    parse_sse_reader(reader, &request_id)
}

/// Non-Unix stub that returns transport errors.
#[cfg(not(unix))]
fn ipc_call(_socket_path: &str, request: EngineRequest) -> EngineResponse {
    let request_id = request.id.clone();
    EngineResponse::failure(
        &request_id,
        transport_error_with_message(
            &request_id,
            "local_ipc",
            "Local IPC transport is only available on Unix platforms",
        ),
    )
}

#[cfg(not(unix))]
fn ipc_stream(_socket_path: &str, request: EngineRequest) -> EngineEventStream {
    let request_id = request.id.clone();
    vec![EngineEvent::Error {
        error: transport_error_with_message(
            &request_id,
            "local_ipc",
            "Local IPC transport is only available on Unix platforms",
        ),
    }]
}

// Helper constructors to avoid repeating request_id borrows

#[cfg(unix)]
fn ipc_connect_error(
    request_id: &str,
    socket_path: &str,
    error: &std::io::Error,
) -> tench_shared_types::EngineError {
    transport_error_with_message(
        request_id,
        "local_ipc",
        format!("Failed to connect to IPC socket {socket_path}: {error}"),
    )
}

#[cfg(unix)]
fn ipc_serialize_error(
    request_id: &str,
    error: &serde_json::Error,
) -> tench_shared_types::EngineError {
    transport_error_with_message(
        request_id,
        "local_ipc",
        format!("Failed to serialize request: {error}"),
    )
}

#[cfg(unix)]
fn ipc_write_error(request_id: &str, error: &std::io::Error) -> tench_shared_types::EngineError {
    transport_error_with_message(
        request_id,
        "local_ipc",
        format!("Failed to write to IPC socket: {error}"),
    )
}

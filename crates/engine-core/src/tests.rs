use std::io::BufReader;

use serde_json::json;
use tench_shared_types::{
    AcceleratorBackend, ChatCompletionParams, ChatMessage, EngineErrorType, EngineEvent,
    EngineMethod, EngineRequest, EngineResponse, LocalRuntimeStatus, ProviderStatus,
};

use super::*;
use crate::providers::ollama_codec::parse_ollama_tags;
use crate::providers::openai_compat::{parse_llama_cpp_models, parse_openai_chat_sse_reader};
use crate::wire::{parse_sse_reader, set_tench_app_id, set_tench_request_id};

#[test]
fn mock_transport_lists_models() {
    let transport = MockTransport::default();
    let response = transport.call(EngineRequest::new(
        "req_1",
        EngineMethod::ModelsList,
        json!({}),
    ));

    assert!(response.error.is_none());
    let result = response.result.expect("models result");
    assert_eq!(result["object"], "list");
    assert_eq!(result["data"][0]["id"], "tench/chat");
}

#[test]
fn mock_chat_completion_returns_echo_response() {
    let transport = MockTransport::default();
    let response = transport.call(EngineRequest::new(
        "req_2",
        EngineMethod::ChatCompletionsCreate,
        json!({
            "model": "tench/chat",
            "messages": [{ "role": "user", "content": "hello" }]
        }),
    ));

    assert!(response.error.is_none());
    let result = response.result.expect("chat result");
    assert_eq!(result["id"], "req_2");
    assert_eq!(
        result["choices"][0]["message"]["content"],
        "mock response: hello"
    );
}

#[test]
fn mock_stream_emits_token_metadata_done() {
    let transport = MockTransport::default();
    let events = transport.stream(EngineRequest::new(
        "req_3",
        EngineMethod::ChatCompletionsCreate,
        json!({
            "model": "tench/chat",
            "messages": [{ "role": "user", "content": "stream me" }],
            "stream": true
        }),
    ));

    assert!(matches!(events.first(), Some(EngineEvent::Token { .. })));
    assert!(matches!(
        events.get(events.len() - 2),
        Some(EngineEvent::Metadata { .. })
    ));
    assert!(matches!(events.last(), Some(EngineEvent::Done { .. })));
}

#[test]
fn unknown_method_returns_contract_error() {
    let router = EngineRouter::default();
    let response = router.call_method("req_4", "unknown.method", json!({}));

    let error = response.error.expect("error");
    assert_eq!(error.code, "invalid_method");
    assert_eq!(error.error_type, EngineErrorType::InvalidRequest);
    assert!(!error.retryable);
}

#[test]
fn missing_model_returns_model_not_found() {
    let transport = MockTransport::default();
    let response = transport.call(EngineRequest::new(
        "req_5",
        EngineMethod::ChatCompletionsCreate,
        json!({
            "model": "cloud/example/missing",
            "messages": []
        }),
    ));

    let error = response.error.expect("error");
    assert_eq!(error.code, "model_not_found");
}

#[test]
fn task_cancel_returns_cancelled_state() {
    let transport = MockTransport::default();
    let response = transport.cancel("task_123");

    assert!(response.error.is_none());
    let result = response.result.expect("cancel result");
    assert_eq!(result["task_id"], "task_123");
    assert_eq!(result["state"], "cancelled");
}

#[test]
fn http_models_maps_to_engine_models_list() {
    let router = EngineRouter::default();
    let response = map_http_compat(&router, "GET", "/v1/models", json!({}));

    assert_eq!(response.status, 200);
    assert_eq!(response.body["data"][0]["id"], "tench/chat");
}

#[test]
fn http_chat_maps_to_engine_chat_completion() {
    let router = EngineRouter::default();
    let response = map_http_compat(
        &router,
        "POST",
        "/v1/chat/completions",
        json!({
            "id": "http_req_1",
            "model": "tench/chat",
            "messages": [{ "role": "user", "content": "from http" }]
        }),
    );

    assert_eq!(response.status, 200);
    assert_eq!(response.body["id"], "http_req_1");
    assert_eq!(
        response.body["choices"][0]["message"]["content"],
        "mock response: from http"
    );
}

#[test]
fn rpc_url_normalizes_trailing_slashes() {
    assert_eq!(
        crate::wire::rpc_url("http://127.0.0.1:1873"),
        "http://127.0.0.1:1873/rpc"
    );
    assert_eq!(
        crate::wire::rpc_url("http://127.0.0.1:1873/"),
        "http://127.0.0.1:1873/rpc"
    );
}

#[test]
fn parse_sse_reader_maps_contract_events() {
    let payload = concat!(
        "event: token\n",
        "data: {\"request_id\":\"req_1\",\"delta\":\"hello \"}\n",
        "\n",
        "event: done\n",
        "data: {\"request_id\":\"req_1\",\"usage\":{\"input_tokens\":1,\"output_tokens\":2,\"total_tokens\":3}}\n",
        "\n",
    );

    let events = parse_sse_reader(BufReader::new(payload.as_bytes()), "req_1");

    assert!(matches!(events.first(), Some(EngineEvent::Token { .. })));
    assert!(matches!(events.last(), Some(EngineEvent::Done { .. })));
}

#[test]
fn auto_provider_uses_native_runtime_not_external_adapter() {
    let provider = LocalProvider::auto();

    assert!(matches!(provider, LocalProvider::Native(_)));
    assert_eq!(provider.provider().id, "native");
    assert_eq!(provider.provider().status, ProviderStatus::NotReady);
    assert_eq!(provider.runtime().status, LocalRuntimeStatus::NotReady);
}

#[test]
fn parse_rocm_smi_json_extracts_amd_gpu_profile() {
    let value = json!({
        "card0": {
            "GPU use (%)": "100",
            "VRAM Total Memory (B)": "103079215104",
            "VRAM Total Used Memory (B)": "84059082752",
            "Card Series": "Radeon 8060S Graphics",
            "Card Vendor": "Advanced Micro Devices, Inc. [AMD/ATI]",
            "GFX Version": "gfx1151"
        },
        "system": {
            "Driver version": "6.17.0-20-generic"
        }
    });

    let accelerators = hardware::parse_rocm_smi_json(&value);

    assert_eq!(accelerators.len(), 1);
    assert_eq!(accelerators[0].name, "Radeon 8060S Graphics");
    assert_eq!(accelerators[0].backend, AcceleratorBackend::Rocm);
    assert_eq!(accelerators[0].gfx_version.as_deref(), Some("gfx1151"));
    assert_eq!(accelerators[0].total_memory_bytes, Some(103079215104));
}

#[test]
fn native_model_scanner_discovers_supported_files() {
    let dir = std::env::temp_dir().join(format!("tench-native-model-scan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tiny.gguf"), b"placeholder").unwrap();
    std::fs::write(dir.join("notes.txt"), b"ignore").unwrap();

    let provider = NativeProvider::new(vec![dir.clone()]);
    let models = provider.list_models();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "local/native/tiny");
    assert_eq!(models[0].provider, "native");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn native_chat_loads_model_catalog_before_executor() {
    let dir = std::env::temp_dir().join(format!("tench-native-chat-model-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tiny.gguf"), b"placeholder").unwrap();

    let provider = NativeProvider::new(vec![dir.clone()]);
    let error = provider
        .chat_completion(
            "req_native",
            ChatCompletionParams {
                model: "local/native/tiny".to_string(),
                messages: vec![ChatMessage::user("hello")],
                stream: false,
            },
        )
        .expect_err("native runtime should require an executor");

    assert_eq!(error.code, "native_runtime_not_ready");
    assert_eq!(error.details["model"], "local/native/tiny");
    assert!(error.details["metadata"]["parse_error"].is_string());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn native_chat_returns_model_not_found_for_unknown_model() {
    let dir =
        std::env::temp_dir().join(format!("tench-native-missing-model-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let provider = NativeProvider::new(vec![dir.clone()]);
    let error = provider
        .chat_completion(
            "req_native_missing",
            ChatCompletionParams {
                model: "local/native/missing".to_string(),
                messages: vec![ChatMessage::user("hello")],
                stream: false,
            },
        )
        .expect_err("model is missing");

    assert_eq!(error.code, "model_not_found");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn parse_ollama_tags_uses_external_runtime_namespace() {
    let models = parse_ollama_tags(json!({
        "models": [{ "name": "qwen2.5-coder:7b" }]
    }));

    assert_eq!(models[0].id, "runtime/ollama/qwen2.5-coder:7b");
    assert_eq!(models[0].provider, "ollama");
}

#[test]
fn parse_llama_cpp_models_uses_external_runtime_namespace() {
    let models = parse_llama_cpp_models(json!({
        "object": "list",
        "data": [
            {
                "id": "qwen2.5-coder-7b",
                "object": "model",
                "owned_by": "llamacpp"
            }
        ]
    }));

    assert_eq!(models[0].id, "runtime/llamacpp/qwen2.5-coder-7b");
    assert_eq!(models[0].provider, "llamacpp");
}

#[test]
fn openai_compatible_stream_maps_tokens_and_done() {
    let data = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}],\"usage\":{\"prompt_tokens\":2,\"completion_tokens\":2}}\n\n",
    );

    let events = parse_openai_chat_sse_reader(
        BufReader::new(data.as_bytes()),
        "req_1",
        "runtime/llamacpp/qwen",
        "llamacpp",
    );

    assert!(matches!(events[0], EngineEvent::Token { .. }));
    assert!(matches!(events[1], EngineEvent::Token { .. }));
    assert!(matches!(events[2], EngineEvent::Metadata { .. }));
    assert!(matches!(events[3], EngineEvent::Done { .. }));
}

// ── Model ID routing tests (#665) ──

#[test]
fn multi_provider_router_routes_tench_alias_to_mock() {
    let router = MultiProviderRouter::with_defaults();
    let response = router.call(EngineRequest::new(
        "req_route_1",
        EngineMethod::ChatCompletionsCreate,
        json!({
            "model": "tench/chat",
            "messages": [{ "role": "user", "content": "hello" }]
        }),
    ));

    assert!(response.error.is_none());
    let result = response.result.expect("result");
    assert_eq!(
        result["choices"][0]["message"]["content"],
        "mock response: hello"
    );
}

#[test]
fn multi_provider_router_routes_cloud_to_fallback() {
    let router = MultiProviderRouter::with_defaults();
    let response = router.call(EngineRequest::new(
        "req_route_cloud",
        EngineMethod::ChatCompletionsCreate,
        json!({
            "model": "cloud/openai/gpt-4o",
            "messages": [{ "role": "user", "content": "hello" }]
        }),
    ));

    // Cloud models aren't registered in mock, so should get model_not_found
    let error = response.error.expect("should error");
    assert_eq!(error.code, "model_not_found");
}

#[test]
fn multi_provider_router_lists_all_models() {
    let router = MultiProviderRouter::with_defaults();
    let response = router.call(EngineRequest::new(
        "req_models",
        EngineMethod::ModelsList,
        json!({}),
    ));

    assert!(response.error.is_none());
    let result = response.result.expect("result");
    let models = result["data"].as_array().expect("models array");
    assert!(!models.is_empty());
    // Mock provider has 3 models
    assert!(models.iter().any(|m| m["id"] == "tench/chat"));
}

#[test]
fn multi_provider_router_cancel_delegates_to_fallback() {
    let router = MultiProviderRouter::with_defaults();
    let response = router.cancel("task_123");

    assert!(response.error.is_none());
    let result = response.result.expect("result");
    assert_eq!(result["task_id"], "task_123");
    assert_eq!(result["state"], "cancelled");
}

// ── Retry-After header tests (#666) ──

#[test]
fn http_compat_response_includes_retry_after_for_rate_limit() {
    let error = tench_shared_types::EngineError::new(
        "rate_limited",
        "Too many requests",
        EngineErrorType::RateLimitError,
        "req_retry_1",
        true,
    );
    let response = EngineResponse::failure("req_retry_1", error);
    let compat = crate::http_compat::http_response_from_engine(response);

    assert_eq!(compat.status, 429);
    assert_eq!(compat.retry_after_seconds, Some(1));
}

#[test]
fn http_compat_response_includes_retry_after_for_provider_error() {
    let error = tench_shared_types::EngineError::new(
        "provider_down",
        "Provider unavailable",
        EngineErrorType::ProviderError,
        "req_retry_2",
        true,
    );
    let response = EngineResponse::failure("req_retry_2", error);
    let compat = crate::http_compat::http_response_from_engine(response);

    assert_eq!(compat.status, 500);
    assert_eq!(compat.retry_after_seconds, Some(5));
}

#[test]
fn http_compat_response_no_retry_after_for_invalid_request() {
    let error = tench_shared_types::EngineError::new(
        "bad_request",
        "Invalid input",
        EngineErrorType::InvalidRequest,
        "req_retry_3",
        false,
    );
    let response = EngineResponse::failure("req_retry_3", error);
    let compat = crate::http_compat::http_response_from_engine(response);

    assert_eq!(compat.status, 400);
    assert_eq!(compat.retry_after_seconds, None);
}

#[test]
fn http_compat_response_no_retry_after_for_success() {
    let response = EngineResponse::success("req_ok", json!({"result": "ok"}));
    let compat = crate::http_compat::http_response_from_engine(response);

    assert_eq!(compat.status, 200);
    assert_eq!(compat.retry_after_seconds, None);
}

// ── Common header injection tests (#665) ──

#[test]
fn thread_local_app_id_can_be_set_and_cleared() {
    set_tench_app_id(Some("tench-docs".to_string()));
    // Value is set in thread-local; verified by reading it back
    let value = crate::wire::TENCH_APP_ID.with(|cell| cell.borrow().clone());
    assert_eq!(value, Some("tench-docs".to_string()));

    set_tench_app_id(None);
    let value = crate::wire::TENCH_APP_ID.with(|cell| cell.borrow().clone());
    assert_eq!(value, None);
}

#[test]
fn thread_local_request_id_can_be_set_and_cleared() {
    set_tench_request_id(Some("req_abc".to_string()));
    let value = crate::wire::TENCH_REQUEST_ID.with(|cell| cell.borrow().clone());
    assert_eq!(value, Some("req_abc".to_string()));

    set_tench_request_id(None);
    let value = crate::wire::TENCH_REQUEST_ID.with(|cell| cell.borrow().clone());
    assert_eq!(value, None);
}

pub mod catalog;
pub mod engine;
pub mod platform;

pub use catalog::*;
pub use engine::*;
pub use platform::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn engine_request_serializes_json_rpc_method_shape() {
        let request = EngineRequest::new(
            "req_1",
            EngineMethod::ChatCompletionsCreate,
            json!({ "model": "tench/chat" }),
        );
        let value = serde_json::to_value(request).expect("request json");

        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["id"], "req_1");
        assert_eq!(value["method"], "chat.completions.create");
        assert_eq!(value["params"]["model"], "tench/chat");
    }

    #[test]
    fn engine_error_serializes_contract_shape() {
        let error = EngineError::new(
            "model_not_found",
            "Model not found",
            EngineErrorType::InvalidRequest,
            "req_2",
            false,
        );
        let value = serde_json::to_value(error).expect("error json");

        assert_eq!(value["code"], "model_not_found");
        assert_eq!(value["message"], "Model not found");
        assert_eq!(value["type"], "invalid_request");
        assert_eq!(value["request_id"], "req_2");
        assert_eq!(value["retryable"], false);
        assert_eq!(value["details"], json!({}));
    }

    #[test]
    fn connection_profile_serializes_transport_kind() {
        let profile = ConnectionProfile::RemotePrivate {
            url: "https://engine.example".to_string(),
            network: PrivateNetwork::Tailscale,
        };
        let value = serde_json::to_value(profile).expect("profile json");

        assert_eq!(value["kind"], "remote_private");
        assert_eq!(value["url"], "https://engine.example");
        assert_eq!(value["network"], "tailscale");
    }

    #[test]
    fn office_products_are_registered() {
        let product_ids: Vec<&str> = PRODUCTS.iter().map(|product| product.slug).collect();

        assert!(product_ids.contains(&"tench-docs"));
        assert!(product_ids.contains(&"tench-sheets"));
        assert!(product_ids.contains(&"tench-slides"));
    }

    // Edge case tests
    #[test]
    fn all_product_role_variants_display() {
        for variant in [
            ProductRole::Foundation,
            ProductRole::Creator,
            ProductRole::Viewer,
            ProductRole::Developer,
            ProductRole::Research,
            ProductRole::Learning,
        ] {
            let label = format!("{}", variant);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn all_engine_method_variants_roundtrip() {
        for variant in [
            EngineMethod::ModelsList,
            EngineMethod::ChatCompletionsCreate,
            EngineMethod::CompletionsCreate,
            EngineMethod::EmbeddingsCreate,
            EngineMethod::ProvidersList,
            EngineMethod::TasksGet,
            EngineMethod::TasksCancel,
            EngineMethod::UsageStats,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: EngineMethod = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
            assert!(!variant.as_str().is_empty());
        }
    }

    #[test]
    fn engine_method_from_str_invalid() {
        assert!("invalid.method".parse::<EngineMethod>().is_err());
        assert!("".parse::<EngineMethod>().is_err());
    }

    #[test]
    fn engine_method_from_str_all_valid() {
        for variant in [
            EngineMethod::ModelsList,
            EngineMethod::ChatCompletionsCreate,
            EngineMethod::CompletionsCreate,
            EngineMethod::EmbeddingsCreate,
            EngineMethod::ProvidersList,
            EngineMethod::TasksGet,
            EngineMethod::TasksCancel,
            EngineMethod::UsageStats,
        ] {
            let parsed: EngineMethod = variant.as_str().parse().unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn all_engine_error_type_variants_roundtrip() {
        for variant in [
            EngineErrorType::InvalidRequest,
            EngineErrorType::AuthenticationError,
            EngineErrorType::PermissionError,
            EngineErrorType::RateLimitError,
            EngineErrorType::ProviderError,
            EngineErrorType::RuntimeError,
            EngineErrorType::ResourceError,
            EngineErrorType::Cancelled,
            EngineErrorType::InternalError,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: EngineErrorType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_private_network_variants_roundtrip() {
        for variant in [
            PrivateNetwork::Tailscale,
            PrivateNetwork::WireGuard,
            PrivateNetwork::Headscale,
            PrivateNetwork::SshTunnel,
            PrivateNetwork::Manual,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: PrivateNetwork = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_model_location_variants_roundtrip() {
        for variant in [
            ModelLocation::Local,
            ModelLocation::Lan,
            ModelLocation::RemotePrivate,
            ModelLocation::RemotePublic,
            ModelLocation::Cloud,
            ModelLocation::Mock,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: ModelLocation = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_provider_status_variants_roundtrip() {
        for variant in [
            ProviderStatus::Available,
            ProviderStatus::Unavailable,
            ProviderStatus::NotConfigured,
            ProviderStatus::NotReady,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: ProviderStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_task_state_variants_roundtrip() {
        for variant in [
            TaskState::Queued,
            TaskState::Running,
            TaskState::Completed,
            TaskState::Failed,
            TaskState::Cancelled,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: TaskState = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_accelerator_backend_variants_roundtrip() {
        for variant in [
            AcceleratorBackend::Rocm,
            AcceleratorBackend::OpenCl,
            AcceleratorBackend::Vulkan,
            AcceleratorBackend::Cuda,
            AcceleratorBackend::Metal,
            AcceleratorBackend::Cpu,
            AcceleratorBackend::Unknown,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: AcceleratorBackend = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_local_runtime_kind_variants_roundtrip() {
        for variant in [
            LocalRuntimeKind::Native,
            LocalRuntimeKind::Ollama,
            LocalRuntimeKind::LlamaCpp,
            LocalRuntimeKind::Mock,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: LocalRuntimeKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_local_runtime_status_variants_roundtrip() {
        for variant in [
            LocalRuntimeStatus::Available,
            LocalRuntimeStatus::Unavailable,
            LocalRuntimeStatus::NotInstalled,
            LocalRuntimeStatus::NotReady,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: LocalRuntimeStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn engine_request_empty_params() {
        let request = EngineRequest::new("req_1", EngineMethod::ModelsList, json!({}));
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: EngineRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(request, deserialized);
    }

    #[test]
    fn engine_response_success_none_result() {
        let response = EngineResponse::success("resp_1", json!({}));
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: EngineResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn engine_response_failure_with_error() {
        let error = EngineError::new(
            "err_1",
            "something went wrong",
            EngineErrorType::InternalError,
            "req_3",
            false,
        );
        let response = EngineResponse::failure("resp_2", error);
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: EngineResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn engine_error_empty_details() {
        let error = EngineError::new("", "", EngineErrorType::InvalidRequest, "", false);
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: EngineError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn engine_error_with_details() {
        let error = EngineError::new("e1", "msg", EngineErrorType::RuntimeError, "r1", true)
            .with_details(json!({ "key": "value" }));
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: EngineError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn connection_profile_all_variants_roundtrip() {
        let variants = [
            ConnectionProfile::Mock {
                name: String::new(),
            },
            ConnectionProfile::LocalIpc {
                endpoint: String::new(),
            },
            ConnectionProfile::LanRpc { url: String::new() },
            ConnectionProfile::RemotePrivate {
                url: String::new(),
                network: PrivateNetwork::WireGuard,
            },
            ConnectionProfile::RemotePublic { url: String::new() },
            ConnectionProfile::HttpCompat {
                base_url: String::new(),
            },
        ];
        for variant in variants {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: ConnectionProfile = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn connection_profile_default_is_mock() {
        let default = ConnectionProfile::default();
        match default {
            ConnectionProfile::Mock { name } => assert_eq!(name, "mock-engine"),
            _ => panic!("default should be Mock"),
        }
    }

    #[test]
    fn model_descriptor_empty_strings() {
        let desc = ModelDescriptor {
            id: String::new(),
            display_name: String::new(),
            provider: String::new(),
            capability: String::new(),
            location: ModelLocation::Mock,
        };
        let serialized = serde_json::to_string(&desc).unwrap();
        let deserialized: ModelDescriptor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, deserialized);
    }

    #[test]
    fn provider_descriptor_empty_strings() {
        let desc = ProviderDescriptor {
            id: String::new(),
            display_name: String::new(),
            status: ProviderStatus::NotReady,
            location: ModelLocation::Cloud,
        };
        let serialized = serde_json::to_string(&desc).unwrap();
        let deserialized: ProviderDescriptor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, deserialized);
    }

    #[test]
    fn task_status_none_progress_and_message() {
        let status = TaskStatus {
            task_id: String::new(),
            state: TaskState::Queued,
            progress: None,
            message: None,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: TaskStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn task_status_zero_progress() {
        let status = TaskStatus {
            task_id: String::new(),
            state: TaskState::Running,
            progress: Some(0.0),
            message: Some(String::new()),
        };
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: TaskStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn usage_stats_zero_tokens() {
        let stats = UsageStats::new(0, 0);
        assert_eq!(stats.input_tokens, 0);
        assert_eq!(stats.output_tokens, 0);
        assert_eq!(stats.total_tokens, 0);
    }

    #[test]
    fn usage_stats_very_large_tokens() {
        let stats = UsageStats::new(u64::MAX / 2, u64::MAX / 2);
        assert_eq!(stats.input_tokens, u64::MAX / 2);
        assert_eq!(stats.output_tokens, u64::MAX / 2);
        assert_eq!(stats.total_tokens, u64::MAX - 1);
    }

    #[test]
    fn hardware_profile_empty_accelerators() {
        let profile = HardwareProfile {
            os: String::new(),
            arch: String::new(),
            accelerators: vec![],
        };
        let serialized = serde_json::to_string(&profile).unwrap();
        let deserialized: HardwareProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn accelerator_descriptor_none_optional_fields() {
        let desc = AcceleratorDescriptor {
            id: String::new(),
            name: String::new(),
            vendor: String::new(),
            backend: AcceleratorBackend::Unknown,
            gfx_version: None,
            total_memory_bytes: None,
            used_memory_bytes: None,
            utilization_percent: None,
        };
        let serialized = serde_json::to_string(&desc).unwrap();
        let deserialized: AcceleratorDescriptor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, deserialized);
    }

    #[test]
    fn accelerator_descriptor_zero_and_max_values() {
        let desc = AcceleratorDescriptor {
            id: String::new(),
            name: String::new(),
            vendor: String::new(),
            backend: AcceleratorBackend::Cpu,
            gfx_version: Some(String::new()),
            total_memory_bytes: Some(0),
            used_memory_bytes: Some(0),
            utilization_percent: Some(0),
        };
        let serialized = serde_json::to_string(&desc).unwrap();
        let deserialized: AcceleratorDescriptor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, deserialized);
    }

    #[test]
    fn accelerator_descriptor_max_values() {
        let desc = AcceleratorDescriptor {
            id: String::new(),
            name: String::new(),
            vendor: String::new(),
            backend: AcceleratorBackend::Cuda,
            gfx_version: Some(String::new()),
            total_memory_bytes: Some(u64::MAX),
            used_memory_bytes: Some(u64::MAX),
            utilization_percent: Some(u32::MAX),
        };
        let serialized = serde_json::to_string(&desc).unwrap();
        let deserialized: AcceleratorDescriptor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, deserialized);
    }

    #[test]
    fn local_runtime_descriptor_none_fields() {
        let desc = LocalRuntimeDescriptor {
            kind: LocalRuntimeKind::Mock,
            status: LocalRuntimeStatus::NotInstalled,
            endpoint: None,
            version: None,
            hardware: None,
        };
        let serialized = serde_json::to_string(&desc).unwrap();
        let deserialized: LocalRuntimeDescriptor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, deserialized);
    }

    #[test]
    fn chat_message_user_empty_content() {
        let msg = ChatMessage::user("");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "");
    }

    #[test]
    fn chat_message_assistant_empty_content() {
        let msg = ChatMessage::assistant("");
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "");
    }

    #[test]
    fn chat_completion_params_empty_messages() {
        let params = ChatCompletionParams {
            model: String::new(),
            messages: vec![],
            stream: false,
        };
        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: ChatCompletionParams = serde_json::from_str(&serialized).unwrap();
        assert_eq!(params, deserialized);
    }

    #[test]
    fn chat_completion_result_empty_choices() {
        let result = ChatCompletionResult {
            id: String::new(),
            model: String::new(),
            choices: vec![],
            usage: UsageStats::new(0, 0),
        };
        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: ChatCompletionResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn chat_choice_empty_finish_reason() {
        let choice = ChatChoice {
            index: 0,
            message: ChatMessage::user(""),
            finish_reason: String::new(),
        };
        let serialized = serde_json::to_string(&choice).unwrap();
        let deserialized: ChatChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(choice, deserialized);
    }

    #[test]
    fn all_products_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for product in PRODUCTS.iter() {
            assert!(
                seen.insert(product.slug),
                "duplicate product slug: {}",
                product.slug
            );
        }
    }

    #[test]
    fn runtime_policy_fields() {
        const _: () = assert!(RUNTIME_POLICY.local_first);
        const _: () = assert!(RUNTIME_POLICY.allows_cloud_fallback);
        const _: () = assert!(!RUNTIME_POLICY.prompt_logging);
        assert!(!RUNTIME_POLICY.metrics_scope.is_empty());
    }

    // ── Model ID tests (#665) ──

    #[test]
    fn model_id_parse_local_native() {
        let (ns, name) = parse_model_id("local/native/qwen2.5-coder-7b").unwrap();
        assert_eq!(ns, ModelNamespace::LocalNative);
        assert_eq!(name, "qwen2.5-coder-7b");
    }

    #[test]
    fn model_id_parse_runtime() {
        let (ns, name) = parse_model_id("runtime/ollama/llama3.1").unwrap();
        assert_eq!(ns, ModelNamespace::Runtime);
        assert_eq!(name, "llama3.1");
    }

    #[test]
    fn model_id_parse_runtime_llamacpp() {
        let (ns, name) = parse_model_id("runtime/llamacpp/qwen2.5-coder-7b").unwrap();
        assert_eq!(ns, ModelNamespace::Runtime);
        assert_eq!(name, "qwen2.5-coder-7b");
    }

    #[test]
    fn model_id_parse_cloud() {
        let (ns, name) = parse_model_id("cloud/openai/gpt-4o-mini").unwrap();
        assert_eq!(ns, ModelNamespace::Cloud);
        assert_eq!(name, "gpt-4o-mini");
    }

    #[test]
    fn model_id_parse_tench_alias() {
        let (ns, name) = parse_model_id("tench/chat").unwrap();
        assert_eq!(ns, ModelNamespace::TenchAlias);
        assert_eq!(name, "chat");
    }

    #[test]
    fn model_id_parse_tench_code_alias() {
        let (ns, name) = parse_model_id("tench/code").unwrap();
        assert_eq!(ns, ModelNamespace::TenchAlias);
        assert_eq!(name, "code");
    }

    #[test]
    fn model_id_parse_invalid_returns_none() {
        assert!(parse_model_id("").is_none());
        assert!(parse_model_id("unknown/model").is_none());
        assert!(parse_model_id("local/native/").is_none());
        assert!(parse_model_id("runtime/").is_none());
        assert!(parse_model_id("runtime/ollama/").is_none());
        assert!(parse_model_id("cloud/").is_none());
        assert!(parse_model_id("cloud/openai/").is_none());
        assert!(parse_model_id("tench/").is_none());
    }

    #[test]
    fn model_id_format_roundtrip() {
        assert_eq!(
            format_model_id(ModelNamespace::LocalNative, ("", "qwen2.5-coder-7b")),
            "local/native/qwen2.5-coder-7b"
        );
        assert_eq!(
            format_model_id(ModelNamespace::Runtime, ("ollama", "llama3.1")),
            "runtime/ollama/llama3.1"
        );
        assert_eq!(
            format_model_id(ModelNamespace::Cloud, ("openai", "gpt-4o-mini")),
            "cloud/openai/gpt-4o-mini"
        );
        assert_eq!(
            format_model_id(ModelNamespace::TenchAlias, ("", "chat")),
            "tench/chat"
        );
    }

    #[test]
    fn model_namespace_variants_roundtrip() {
        for variant in [
            ModelNamespace::LocalNative,
            ModelNamespace::Runtime,
            ModelNamespace::Cloud,
            ModelNamespace::TenchAlias,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: ModelNamespace = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    // ── Retryable error type tests (#666) ──

    #[test]
    fn retryable_error_types() {
        assert!(is_retryable_error_type(&EngineErrorType::ProviderError));
        assert!(is_retryable_error_type(&EngineErrorType::RateLimitError));
        assert!(is_retryable_error_type(&EngineErrorType::InternalError));
    }

    #[test]
    fn non_retryable_error_types() {
        assert!(!is_retryable_error_type(&EngineErrorType::InvalidRequest));
        assert!(!is_retryable_error_type(
            &EngineErrorType::AuthenticationError
        ));
        assert!(!is_retryable_error_type(&EngineErrorType::PermissionError));
        assert!(!is_retryable_error_type(&EngineErrorType::RuntimeError));
        assert!(!is_retryable_error_type(&EngineErrorType::ResourceError));
        assert!(!is_retryable_error_type(&EngineErrorType::Cancelled));
    }

    // ── Progress event payload tests (#667) ──

    #[test]
    fn progress_event_with_eta_ms_and_stage() {
        let event = EngineEvent::Progress {
            task_id: "task_123".to_string(),
            request_id: "req_123".to_string(),
            stage: "transcribing".to_string(),
            progress: 0.42,
            message: "Processing audio".to_string(),
            eta_ms: Some(12000),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: EngineEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);

        let value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value["event"], "progress");
        assert_eq!(value["data"]["stage"], "transcribing");
        assert_eq!(value["data"]["eta_ms"], 12000);
    }

    #[test]
    fn progress_event_without_eta_ms() {
        let event = EngineEvent::Progress {
            task_id: "task_456".to_string(),
            request_id: "req_456".to_string(),
            stage: "downloading".to_string(),
            progress: 0.1,
            message: "Downloading model".to_string(),
            eta_ms: None,
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: EngineEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);

        let value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value["data"]["eta_ms"], Value::Null);
    }
}

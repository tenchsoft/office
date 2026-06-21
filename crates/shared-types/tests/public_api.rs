use serde_json::json;
use tench_shared_types::{
    catalog, engine, EngineError, EngineErrorType, EngineMethod, EngineRequest, ProductRole,
    PRODUCTS, RUNTIME_POLICY,
};

#[test]
fn root_reexports_legacy_catalog_and_engine_types() {
    let method = EngineMethod::ChatCompletionsCreate;
    let request = EngineRequest::new("req_1", method, json!({ "model": "tench/chat" }));
    let error = EngineError::new(
        "provider_error",
        "provider failed",
        EngineErrorType::ProviderError,
        "req_1",
        true,
    );

    assert_eq!(request.method.as_str(), "chat.completions.create");
    assert!(error.retryable);
    assert_eq!(ProductRole::Creator.to_string(), "creator");
    assert!(PRODUCTS.iter().any(|product| product.slug == "tench-docs"));
    const _: () = assert!(RUNTIME_POLICY.local_first);
}

#[test]
fn canonical_modules_export_the_same_public_types() {
    let root_method = EngineMethod::ModelsList;
    let module_method = engine::EngineMethod::ModelsList;
    assert_eq!(root_method, module_method);

    let root_role = ProductRole::Viewer;
    let module_role = catalog::ProductRole::Viewer;
    assert_eq!(root_role, module_role);
}

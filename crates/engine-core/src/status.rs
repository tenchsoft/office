use serde::Serialize;
use tench_shared_types::{ConnectionProfile, TenchProduct, PRODUCTS};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EngineStatus {
    pub runtime_state: &'static str,
    pub products_registered: usize,
    pub localhost_only: bool,
    pub streaming_required: bool,
    pub interface_protocol: &'static str,
    pub preferred_local_transport: &'static str,
    pub remote_transport: &'static str,
    pub http_compatibility: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RoutePreview {
    pub product: TenchProduct,
    pub connection_profile: ConnectionProfile,
    pub engine_endpoint: &'static str,
    pub data_boundary: &'static str,
}

pub fn status() -> EngineStatus {
    EngineStatus {
        runtime_state: "m0-m1-native-control-plane",
        products_registered: PRODUCTS.len(),
        localhost_only: true,
        streaming_required: true,
        interface_protocol: "json-rpc-2.0",
        preferred_local_transport: "ipc",
        remote_transport: "https-json-rpc-sse",
        http_compatibility: true,
    }
}

pub fn products() -> &'static [TenchProduct] {
    &PRODUCTS
}

pub fn route_preview(slug: &str) -> Option<RoutePreview> {
    let product = PRODUCTS
        .iter()
        .copied()
        .find(|product| product.slug == slug)?;

    Some(RoutePreview {
        product,
        connection_profile: ConnectionProfile::LocalIpc {
            endpoint: "tench-engine.local".to_string(),
        },
        engine_endpoint: "engine://local-ipc",
        data_boundary: "user work data stays local unless the user selects a cloud provider",
    })
}

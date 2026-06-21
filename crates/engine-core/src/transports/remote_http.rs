use tench_shared_types::{ConnectionProfile, EngineRequest, EngineResponse};

use crate::wire::{rpc_url, transport_error_with_message, transport_unavailable};
use crate::{EngineClient, EngineEventStream, EngineTransport};

use super::stream::remote_sse_stream;

#[derive(Clone, Debug)]
pub struct RemoteHttpTransport {
    url: String,
}

impl RemoteHttpTransport {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl EngineClient for RemoteHttpTransport {
    fn call(&self, request: EngineRequest) -> EngineResponse {
        let request_id = request.id.clone();
        let url = rpc_url(&self.url);
        let payload = match serde_json::to_value(&request) {
            Ok(payload) => payload,
            Err(error) => {
                return EngineResponse::failure(
                    request_id.clone(),
                    transport_error_with_message(
                        &request_id,
                        "remote_http",
                        format!("Failed to serialize request: {error}"),
                    ),
                )
            }
        };

        match ureq::post(&url).send_json(payload) {
            Ok(response) => response
                .into_json::<EngineResponse>()
                .unwrap_or_else(|error| {
                    EngineResponse::failure(
                        request_id.clone(),
                        transport_error_with_message(
                            &request_id,
                            "remote_http",
                            format!("Failed to parse Engine response: {error}"),
                        ),
                    )
                }),
            Err(ureq::Error::Status(_, response)) => response
                .into_json::<EngineResponse>()
                .unwrap_or_else(|error| {
                    EngineResponse::failure(
                        request_id.clone(),
                        transport_error_with_message(
                            &request_id,
                            "remote_http",
                            format!("Failed to parse Engine error response: {error}"),
                        ),
                    )
                }),
            Err(error) => EngineResponse::failure(
                request_id.clone(),
                transport_error_with_message(&request_id, "remote_http", error.to_string()),
            ),
        }
    }

    fn stream(&self, request: EngineRequest) -> EngineEventStream {
        remote_sse_stream(&self.url, request, "remote_http")
    }

    fn cancel(&self, task_id: &str) -> EngineResponse {
        transport_unavailable(&format!("cancel_{task_id}"), "remote_http")
    }
}

impl EngineTransport for RemoteHttpTransport {
    fn profile(&self) -> ConnectionProfile {
        ConnectionProfile::RemotePublic {
            url: self.url.clone(),
        }
    }
}

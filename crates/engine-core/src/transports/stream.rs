use std::io::BufReader;

use serde_json::json;
use tench_shared_types::{EngineEvent, EngineMethod, EngineRequest};

use crate::wire::{
    endpoint_url, parse_sse_reader, response_to_events, transport_error_with_message,
};
use crate::{EngineClient, EngineEventStream};

use super::remote_http::RemoteHttpTransport;

pub(super) fn remote_sse_stream(
    base_url: &str,
    request: EngineRequest,
    transport: &'static str,
) -> EngineEventStream {
    let request_id = request.id.clone();
    if request.method != EngineMethod::ChatCompletionsCreate {
        let response = RemoteHttpTransport::new(base_url).call(request);
        return response_to_events(response);
    }

    let mut payload = request.params;
    if let Some(object) = payload.as_object_mut() {
        object.insert("id".to_string(), json!(request_id.clone()));
        object.insert("stream".to_string(), json!(true));
    }

    match ureq::post(&endpoint_url(base_url, "/v1/chat/completions")).send_json(payload) {
        Ok(response) => parse_sse_reader(BufReader::new(response.into_reader()), &request_id),
        Err(error) => vec![EngineEvent::Error {
            error: transport_error_with_message(&request_id, transport, error.to_string()),
        }],
    }
}

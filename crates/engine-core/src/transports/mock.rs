use tench_shared_types::{ConnectionProfile, EngineRequest, EngineResponse};

use crate::providers::MockProvider;
use crate::{EngineClient, EngineEventStream, EngineRouter, EngineTransport};

#[derive(Clone, Debug, Default)]
pub struct MockTransport {
    router: EngineRouter<MockProvider>,
}

impl EngineClient for MockTransport {
    fn call(&self, request: EngineRequest) -> EngineResponse {
        self.router.call(request)
    }

    fn stream(&self, request: EngineRequest) -> EngineEventStream {
        self.router.stream(request)
    }

    fn cancel(&self, task_id: &str) -> EngineResponse {
        self.router.cancel(task_id)
    }
}

impl EngineTransport for MockTransport {
    fn profile(&self) -> ConnectionProfile {
        ConnectionProfile::Mock {
            name: "mock-engine".to_string(),
        }
    }
}

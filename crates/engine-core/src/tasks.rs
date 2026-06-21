use serde_json::Value;
use tench_shared_types::{EngineError, EngineErrorType, TaskState, TaskStatus};

pub(crate) fn task_status_from_params(
    request_id: &str,
    params: Value,
    state: TaskState,
) -> Result<TaskStatus, EngineError> {
    let Some(task_id) = params.get("task_id").and_then(Value::as_str) else {
        return Err(EngineError::new(
            "invalid_request",
            "Missing task_id",
            EngineErrorType::InvalidRequest,
            request_id,
            false,
        ));
    };

    Ok(TaskStatus {
        task_id: task_id.to_string(),
        state,
        progress: Some(if state == TaskState::Completed {
            1.0
        } else {
            0.0
        }),
        message: Some(
            match state {
                TaskState::Completed => "task completed",
                TaskState::Cancelled => "task cancelled",
                _ => "task updated",
            }
            .to_string(),
        ),
    })
}

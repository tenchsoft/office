//! Tool/Function Calling Framework (#462)
//!
//! Defines tools that the engine can invoke on behalf of an AI model,
//! including JSON schema validation and a dispatch registry.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Describes a single parameter accepted by a tool.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
}

/// A tool definition that an AI model can request to call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParam>,
}

impl ToolDefinition {
    /// Build a JSON Schema object describing this tool's parameters.
    pub fn json_schema(&self) -> Value {
        let properties: HashMap<&str, Value> = self
            .parameters
            .iter()
            .map(|p| {
                let mut schema = serde_json::json!({
                    "type": p.param_type,
                });
                if let Some(ref desc) = p.description {
                    schema["description"] = Value::String(desc.clone());
                }
                if let Some(ref default) = p.default {
                    schema["default"] = default.clone();
                }
                (p.name.as_str(), schema)
            })
            .collect();

        let required: Vec<&str> = self
            .parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.as_str())
            .collect();

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }

    /// Validate that the supplied arguments match this tool's schema.
    pub fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        let map = args
            .as_object()
            .ok_or_else(|| ToolError::InvalidArgs("arguments must be a JSON object".into()))?;

        // Check required params are present
        for param in &self.parameters {
            if param.required && !map.contains_key(&param.name) {
                return Err(ToolError::MissingParam(param.name.clone()));
            }
            if let Some(val) = map.get(&param.name) {
                validate_type(&param.param_type, val, &param.name)?;
            }
        }
        Ok(())
    }
}

fn validate_type(expected: &str, value: &Value, name: &str) -> Result<(), ToolError> {
    let ok = match expected {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => true, // unknown types pass through
    };
    if ok {
        Ok(())
    } else {
        Err(ToolError::TypeMismatch {
            param: name.to_string(),
            expected: expected.to_string(),
            got: value_type_name(value),
        })
    }
}

fn value_type_name(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(_) => "boolean".into(),
        Value::Number(_) => "number".into(),
        Value::String(_) => "string".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
    }
}

/// Errors that can occur during tool invocation.
#[derive(Clone, Debug)]
pub enum ToolError {
    NotFound(String),
    MissingParam(String),
    InvalidArgs(String),
    TypeMismatch {
        param: String,
        expected: String,
        got: String,
    },
    ExecutionFailed(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "tool not found: {name}"),
            Self::MissingParam(name) => write!(f, "missing required parameter: {name}"),
            Self::InvalidArgs(msg) => write!(f, "invalid arguments: {msg}"),
            Self::TypeMismatch {
                param,
                expected,
                got,
            } => {
                write!(
                    f,
                    "type mismatch for param '{param}': expected {expected}, got {got}"
                )
            }
            Self::ExecutionFailed(msg) => write!(f, "execution failed: {msg}"),
        }
    }
}

impl std::error::Error for ToolError {}

/// The result of a tool invocation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: Value,
}

/// A callable tool handler.
pub type ToolHandler = Box<dyn Fn(&Value) -> Result<Value, ToolError> + Send + Sync>;

/// Registry that maps tool names to definitions and handlers.
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    handlers: HashMap<String, ToolHandler>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// Register a tool with its definition and handler function.
    pub fn register<F>(&mut self, def: ToolDefinition, handler: F)
    where
        F: Fn(&Value) -> Result<Value, ToolError> + Send + Sync + 'static,
    {
        self.handlers.insert(def.name.clone(), Box::new(handler));
        self.tools.insert(def.name.clone(), def);
    }

    /// Look up a tool definition by name.
    pub fn get_definition(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// List all registered tool definitions.
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Validate and dispatch a tool call.
    pub fn dispatch(&self, tool_name: &str, args: &Value) -> Result<ToolResult, ToolError> {
        let def = self
            .tools
            .get(tool_name)
            .ok_or_else(|| ToolError::NotFound(tool_name.to_string()))?;
        def.validate_args(args)?;

        let handler = self
            .handlers
            .get(tool_name)
            .ok_or_else(|| ToolError::NotFound(tool_name.to_string()))?;

        match handler(args) {
            Ok(output) => Ok(ToolResult {
                tool_name: tool_name.to_string(),
                success: true,
                output,
            }),
            Err(e) => Ok(ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: Value::String(e.to_string()),
            }),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_required_param_missing() {
        let tool = ToolDefinition {
            name: "search".into(),
            description: "Search the web".into(),
            parameters: vec![ToolParam {
                name: "query".into(),
                description: None,
                param_type: "string".into(),
                required: true,
                default: None,
            }],
        };
        let result = tool.validate_args(&json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn validate_type_mismatch() {
        let tool = ToolDefinition {
            name: "calc".into(),
            description: "Calculate".into(),
            parameters: vec![ToolParam {
                name: "x".into(),
                description: None,
                param_type: "integer".into(),
                required: true,
                default: None,
            }],
        };
        let result = tool.validate_args(&json!({"x": "hello"}));
        assert!(matches!(result, Err(ToolError::TypeMismatch { .. })));
    }

    #[test]
    fn dispatch_tool_success() {
        let mut registry = ToolRegistry::new();
        let def = ToolDefinition {
            name: "echo".into(),
            description: "Echo input".into(),
            parameters: vec![ToolParam {
                name: "msg".into(),
                description: None,
                param_type: "string".into(),
                required: true,
                default: None,
            }],
        };
        registry.register(def, |args| {
            let msg = args["msg"].as_str().unwrap_or("").to_string();
            Ok(Value::String(msg))
        });

        let result = registry.dispatch("echo", &json!({"msg": "hello"})).unwrap();
        assert!(result.success);
        assert_eq!(result.output, json!("hello"));
    }

    #[test]
    fn dispatch_tool_not_found() {
        let registry = ToolRegistry::new();
        let result = registry.dispatch("missing", &json!({}));
        assert!(matches!(result, Err(ToolError::NotFound(_))));
    }
}

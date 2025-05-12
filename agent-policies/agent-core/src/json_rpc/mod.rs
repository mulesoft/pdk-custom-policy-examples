use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// A JSONRPC request object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<'a> {
    /// The name of the RPC call.
    pub method: &'a str,
    /// Parameters to the RPC call.
    pub params: Option<&'a RawValue>,
    /// Identifier for this request, which should appear in the response.
    pub id: Option<serde_json::Value>,
    /// jsonrpc field, MUST be "2.0".
    pub jsonrpc: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse<'a> {
    /// A result if there is one, or [`None`].
    pub result: Option<&'a RawValue>,
    /// An error if there is one, or [`None`].
    pub error: Option<RpcError>,
    /// Identifier for this response, which should match that of the request.
    pub id: Option<serde_json::Value>,
    /// jsonrpc field, MUST be "2.0".
    pub jsonrpc: Option<&'a str>,
}

/// A JSONRPC error object
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpcError {
    /// The integer identifier of the error
    pub code: i32,
    /// A string describing the error
    pub message: String,
    /// Additional data specific to the error
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    pub fn invalid_json(data: String) -> RpcError {
        RpcError {
            code: -32700,
            message: "Invalid JSON data".to_string(),
            data: Some(serde_json::Value::String(data)),
        }
    }

    pub fn invalid_methods(invalid_method: String, valid_methods: Vec<&str>) -> RpcError {
        let error_message = format!(
            "Invalid methods: `{}`. Valid Methods: {}",
            invalid_method,
            valid_methods.join(", ")
        );

        RpcError {
            code: -32600,
            message: "Request payload validation error".to_string(),
            data: Some(serde_json::Value::String(error_message)),
        }
    }

    pub fn invalid_param(error: String) -> RpcError {
        RpcError {
            code: -32600,
            message: "Request payload validation error".to_string(),
            data: Some(serde_json::Value::String(error)),
        }
    }
}

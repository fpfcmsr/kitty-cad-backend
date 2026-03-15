use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modeling_cmd::ModelingCmd;
use crate::responses::OkModelingCmdResponse;

/// Incoming WebSocket message types from the client.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketRequest {
    /// Authentication headers (first message from client).
    Headers {
        headers: HashMap<String, String>,
    },
    TrickleIce {
        candidate: serde_json::Value,
    },
    SdpOffer {
        offer: serde_json::Value,
    },
    ModelingCmdReq {
        cmd: ModelingCmd,
        cmd_id: Uuid,
    },
    ModelingCmdBatchReq {
        requests: Vec<ModelingCmdReqBatch>,
        #[serde(default)]
        batch_id: Option<Uuid>,
        #[serde(default)]
        responses: Option<bool>,
    },
    /// Client sends ping, expects pong back.
    Ping {},
    /// Client sends pong (keepalive response).
    Pong {},
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct ModelingCmdReqBatch {
    pub cmd: ModelingCmd,
    pub cmd_id: Uuid,
}

/// Outgoing WebSocket response envelope matching Zoo's format.
/// Format: { success: bool, request_id: UUID|null, resp: { type: ..., data: ... } }
#[derive(Debug, Serialize)]
pub struct WebSocketResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resp: Option<OkWebSocketResponseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error_code: String,
    pub message: String,
}

/// Response data types matching Zoo's OkWebSocketResponseData enum.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OkWebSocketResponseData {
    ModelingSessionData {
        data: ModelingSessionDataInner,
    },
    IceServerInfo {
        data: IceServerInfoData,
    },
    SdpAnswer {
        data: SdpAnswer,
    },
    Pong {},
    Modeling {
        data: ModelingResponseData,
    },
    ModelingBatch {
        data: ModelingBatchResponseData,
    },
    Export {
        data: ExportResponseData,
    },
}

#[derive(Debug, Serialize)]
pub struct ModelingSessionDataInner {
    pub api_call_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct IceServerInfoData {
    pub ice_servers: Vec<IceServer>,
}

#[derive(Debug, Serialize)]
pub struct IceServer {
    pub urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SdpAnswer {
    #[serde(rename = "type")]
    pub sdp_type: String,
    pub sdp: String,
}

/// Wraps a modeling command response.
#[derive(Debug, Serialize)]
pub struct ModelingResponseData {
    pub modeling_response: OkModelingCmdResponse,
}

/// Batch response: map of cmd_id -> individual response.
#[derive(Debug, Serialize)]
pub struct ModelingBatchResponseData {
    pub responses: HashMap<Uuid, BatchItemResponse>,
}

#[derive(Debug, Serialize)]
pub struct BatchItemResponse {
    #[serde(flatten)]
    pub response: OkModelingCmdResponse,
}

/// Export response data (files).
#[derive(Debug, Serialize)]
pub struct ExportResponseData {
    pub files: Vec<ExportFileEntry>,
}

#[derive(Debug, Serialize)]
pub struct ExportFileEntry {
    pub name: String,
    pub contents: Vec<u8>,
}

// -- Helper constructors --

impl WebSocketResponse {
    /// Create a success response with no request_id (for server-initiated messages).
    pub fn server_success(resp: OkWebSocketResponseData) -> Self {
        Self {
            success: true,
            request_id: None,
            resp: Some(resp),
            errors: None,
        }
    }

    /// Create a success response for a client request.
    pub fn success(request_id: Uuid, resp: OkWebSocketResponseData) -> Self {
        Self {
            success: true,
            request_id: Some(request_id),
            resp: Some(resp),
            errors: None,
        }
    }

    /// Create a modeling command success response.
    pub fn modeling_success(cmd_id: Uuid, resp: OkModelingCmdResponse) -> Self {
        Self::success(
            cmd_id,
            OkWebSocketResponseData::Modeling {
                data: ModelingResponseData {
                    modeling_response: resp,
                },
            },
        )
    }

    /// Create an error response.
    pub fn error(request_id: Option<Uuid>, message: String) -> Self {
        Self {
            success: false,
            request_id,
            resp: None,
            errors: Some(vec![ApiError {
                error_code: "internal_error".to_string(),
                message,
            }]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_start_path() {
        let json = r#"{"type":"modeling_cmd_req","cmd_id":"00000000-0000-0000-0000-000000000001","cmd":{"type":"start_path"}}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        match req {
            WebSocketRequest::ModelingCmdReq { cmd_id, cmd } => {
                assert_eq!(cmd_id.to_string(), "00000000-0000-0000-0000-000000000001");
                assert!(matches!(cmd, ModelingCmd::StartPath {}));
            }
            _ => panic!("Expected ModelingCmdReq"),
        }
    }

    #[test]
    fn test_deserialize_extrude() {
        let json = r#"{"type":"modeling_cmd_req","cmd_id":"00000000-0000-0000-0000-000000000002","cmd":{"type":"extrude","target":"00000000-0000-0000-0000-000000000001","distance":5.0}}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        match req {
            WebSocketRequest::ModelingCmdReq { cmd, .. } => {
                match cmd {
                    ModelingCmd::Extrude { target, distance, .. } => {
                        assert_eq!(target.to_string(), "00000000-0000-0000-0000-000000000001");
                        assert!((distance - 5.0).abs() < f64::EPSILON);
                    }
                    _ => panic!("Expected Extrude"),
                }
            }
            _ => panic!("Expected ModelingCmdReq"),
        }
    }

    #[test]
    fn test_deserialize_headers() {
        let json = r#"{"type":"headers","headers":{"Authorization":"Bearer test-token"}}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        match req {
            WebSocketRequest::Headers { headers } => {
                assert_eq!(headers.get("Authorization").unwrap(), "Bearer test-token");
            }
            _ => panic!("Expected Headers"),
        }
    }

    #[test]
    fn test_deserialize_ping() {
        let json = r#"{"type":"ping"}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        assert!(matches!(req, WebSocketRequest::Ping {}));
    }

    #[test]
    fn test_deserialize_pong() {
        let json = r#"{"type":"pong"}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        assert!(matches!(req, WebSocketRequest::Pong {}));
    }

    #[test]
    fn test_deserialize_unknown_type() {
        let json = r#"{"type":"some_future_message","data":123}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        assert!(matches!(req, WebSocketRequest::Unknown));
    }

    #[test]
    fn test_deserialize_batch_request() {
        let json = r#"{"type":"modeling_cmd_batch_req","requests":[{"cmd_id":"00000000-0000-0000-0000-000000000001","cmd":{"type":"start_path"}},{"cmd_id":"00000000-0000-0000-0000-000000000002","cmd":{"type":"scene_clear_all"}}]}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        match req {
            WebSocketRequest::ModelingCmdBatchReq { requests, .. } => {
                assert_eq!(requests.len(), 2);
            }
            _ => panic!("Expected ModelingCmdBatchReq"),
        }
    }

    #[test]
    fn test_deserialize_extend_path_line() {
        let json = r#"{"type":"modeling_cmd_req","cmd_id":"00000000-0000-0000-0000-000000000001","cmd":{"type":"extend_path","path":"00000000-0000-0000-0000-000000000002","segment":{"type":"line","end":{"x":10,"y":0,"z":0},"relative":false}}}"#;
        let req: WebSocketRequest = serde_json::from_str(json).unwrap();
        match req {
            WebSocketRequest::ModelingCmdReq { cmd, .. } => {
                assert!(matches!(cmd, ModelingCmd::ExtendPath { .. }));
            }
            _ => panic!("Expected ModelingCmdReq"),
        }
    }

    #[test]
    fn test_serialize_success_response() {
        let resp = WebSocketResponse::modeling_success(
            Uuid::nil(),
            crate::responses::OkModelingCmdResponse::Empty {},
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"modeling\""));
    }

    #[test]
    fn test_serialize_ice_server_info() {
        let resp = WebSocketResponse::server_success(OkWebSocketResponseData::IceServerInfo {
            data: IceServerInfoData {
                ice_servers: vec![],
            },
        });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("ice_server_info"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_serialize_error_response() {
        let resp = WebSocketResponse::error(Some(Uuid::nil()), "test error".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("test error"));
    }
}

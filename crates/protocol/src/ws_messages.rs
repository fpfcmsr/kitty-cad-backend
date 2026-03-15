use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modeling_cmd::ModelingCmd;
use crate::responses::OkModelingCmdResponse;

/// Incoming WebSocket message types from the client.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketRequest {
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
    },
    Pong {},
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct ModelingCmdReqBatch {
    pub cmd: ModelingCmd,
    pub cmd_id: Uuid,
}

/// Outgoing WebSocket message types to the client.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketResponse {
    IceServerInfo {
        ice_servers: Vec<IceServer>,
    },
    SdpAnswer {
        answer: SdpAnswer,
    },
    Pong {},
    Modeling {
        #[serde(flatten)]
        result: ModelingSessionResult,
    },
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

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelingSessionResult {
    Success {
        cmd_id: Uuid,
        resp: OkModelingCmdResponse,
    },
    Error {
        cmd_id: Uuid,
        errors: Vec<String>,
    },
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
            WebSocketRequest::ModelingCmdBatchReq { requests } => {
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
    fn test_serialize_ice_server_info() {
        let resp = WebSocketResponse::IceServerInfo { ice_servers: vec![] };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("ice_server_info"));
    }

    #[test]
    fn test_serialize_success_response() {
        let resp = WebSocketResponse::Modeling {
            result: ModelingSessionResult::Success {
                cmd_id: Uuid::nil(),
                resp: crate::responses::OkModelingCmdResponse::Empty {},
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("success"));
    }
}

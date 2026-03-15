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

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use futures::{SinkExt, StreamExt};

use engine::commands::dispatch;
use engine::session::Session;
use protocol::ws_messages::*;

pub fn router() -> Router {
    Router::new()
        .route("/ws/modeling/commands", get(ws_upgrade))
        .route("/ws/modeling/commands/", get(ws_upgrade))
}

async fn ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_ws)
}

async fn handle_ws(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial ICE server info (empty -- no WebRTC needed for local)
    let ice_msg = WebSocketResponse::IceServerInfo {
        ice_servers: vec![],
    };
    if let Ok(json) = serde_json::to_string(&ice_msg) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    let mut session = Session::new();

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let responses = handle_text_message(&mut session, &text);
                for resp in responses {
                    if let Ok(json) = serde_json::to_string(&resp) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            return;
                        }
                    }
                }
            }
            Message::Binary(data) => {
                // Try to parse binary as JSON text
                if let Ok(text) = String::from_utf8(data.to_vec()) {
                    let responses = handle_text_message(&mut session, &text);
                    for resp in responses {
                        if let Ok(json) = serde_json::to_string(&resp) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                return;
                            }
                        }
                    }
                }
            }
            Message::Ping(data) => {
                let _ = sender.send(Message::Pong(data)).await;
            }
            Message::Close(_) => return,
            _ => {}
        }
    }
}

fn handle_text_message(session: &mut Session, text: &str) -> Vec<WebSocketResponse> {
    let request: WebSocketRequest = match serde_json::from_str(text) {
        Ok(req) => req,
        Err(e) => {
            tracing::warn!("Failed to parse WebSocket message: {e}");
            tracing::debug!("Raw message: {text}");
            return vec![];
        }
    };

    match request {
        WebSocketRequest::Pong {} => vec![],

        WebSocketRequest::TrickleIce { .. } => {
            // No-op for local backend (no WebRTC)
            vec![]
        }

        WebSocketRequest::SdpOffer { .. } => {
            // Return a fake SDP answer so the client doesn't hang
            vec![WebSocketResponse::SdpAnswer {
                answer: SdpAnswer {
                    sdp_type: "answer".to_string(),
                    sdp: "v=0\r\n".to_string(),
                },
            }]
        }

        WebSocketRequest::ModelingCmdReq { cmd, cmd_id } => {
            vec![execute_cmd(session, cmd_id, cmd)]
        }

        WebSocketRequest::ModelingCmdBatchReq { requests } => {
            requests
                .into_iter()
                .map(|req| execute_cmd(session, req.cmd_id, req.cmd))
                .collect()
        }

        WebSocketRequest::Unknown => {
            tracing::debug!("Received unknown WebSocket message type");
            vec![]
        }
    }
}

fn execute_cmd(
    session: &mut Session,
    cmd_id: uuid::Uuid,
    cmd: protocol::modeling_cmd::ModelingCmd,
) -> WebSocketResponse {
    match dispatch(session, cmd) {
        Ok(resp) => WebSocketResponse::Modeling {
            result: ModelingSessionResult::Success { cmd_id, resp },
        },
        Err(e) => WebSocketResponse::Modeling {
            result: ModelingSessionResult::Error {
                cmd_id,
                errors: vec![e],
            },
        },
    }
}

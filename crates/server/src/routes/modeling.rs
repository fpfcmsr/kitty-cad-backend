use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};

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

/// Message sent from the async WebSocket handler to the blocking engine thread.
struct EngineRequest {
    text: String,
    reply: oneshot::Sender<Vec<WebSocketResponse>>,
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

    // Create a channel for communicating with the engine thread.
    // The engine thread owns the Session (which contains non-Send OCCT shapes).
    let (engine_tx, mut engine_rx) = mpsc::channel::<EngineRequest>(32);

    // Spawn a dedicated OS thread for engine operations (non-Send types).
    std::thread::spawn(move || {
        let mut session = Session::new();

        while let Some(req) = engine_rx.blocking_recv() {
            let responses = handle_text_message(&mut session, &req.text);
            let _ = req.reply.send(responses);
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Some(responses) = dispatch_to_engine(&engine_tx, text.to_string()).await {
                    for resp in responses {
                        if let Ok(json) = serde_json::to_string(&resp) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                return;
                            }
                        }
                    }
                }
            }
            Message::Binary(data) => {
                if let Ok(text) = String::from_utf8(data.to_vec()) {
                    if let Some(responses) = dispatch_to_engine(&engine_tx, text).await {
                        for resp in responses {
                            if let Ok(json) = serde_json::to_string(&resp) {
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    return;
                                }
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

/// Send a text message to the engine thread and await the response.
async fn dispatch_to_engine(
    tx: &mpsc::Sender<EngineRequest>,
    text: String,
) -> Option<Vec<WebSocketResponse>> {
    let (reply_tx, reply_rx) = oneshot::channel();
    let req = EngineRequest {
        text,
        reply: reply_tx,
    };
    tx.send(req).await.ok()?;
    reply_rx.await.ok()
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

        WebSocketRequest::ModelingCmdBatchReq { requests } => requests
            .into_iter()
            .map(|req| execute_cmd(session, req.cmd_id, req.cmd))
            .collect(),

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

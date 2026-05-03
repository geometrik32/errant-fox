use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::state::AppState;

// ── DTO types ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct WsCommentAuthor {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WsComment {
    pub id: i32,
    pub video_id: String,
    pub author: WsCommentAuthor,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<String>,
    pub bout_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WsBout {
    pub id: i32,
    pub video_id: String,
    pub order_index: i32,
    pub time_start_ms: i32,
    pub time_end_ms: i32,
    pub score_a: i32,
    pub score_b: i32,
    pub technique_a_id: Option<i32>,
    pub technique_b_id: Option<i32>,
    pub hit_zone_a: Option<String>,
    pub hit_zone_b: Option<String>,
    pub result_a: Option<String>,
    pub result_b: Option<String>,
}

// ── WsEvent ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    NewComment(WsComment),
    UpdateBout(WsBout),
    NewVideo {
        id: String,
        date: String,
        preview_url: String,
    },
}

impl WsEvent {
    pub fn video_id(&self) -> Option<&str> {
        match self {
            WsEvent::NewComment(c) => Some(&c.video_id),
            WsEvent::UpdateBout(b) => Some(&b.video_id),
            WsEvent::NewVideo { .. } => None,
        }
    }
}

pub type WsHub = broadcast::Sender<WsEvent>;

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // First message must be {"token": "..."}
    let _user_id = match socket.recv().await {
        Some(Ok(Message::Text(text))) => {
            let val: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => return,
            };
            let token = match val.get("token").and_then(|t| t.as_str()) {
                Some(t) => t.to_string(),
                None => return,
            };
            match crate::api::auth::verify_token(&token, &state.jwt_secret) {
                Ok(claims) => claims.sub,
                Err(_) => return,
            }
        }
        _ => return,
    };

    let mut rx = state.ws_hub.subscribe();
    let mut watching: Option<String> = None;

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(vid) = val.get("watching").and_then(|v| v.as_str()) {
                                watching = Some(vid.to_string());
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        let should_send = match ev.video_id() {
                            Some(vid) => watching.as_deref() == Some(vid),
                            None => true,
                        };
                        if should_send {
                            if let Ok(json) = serde_json::to_string(&ev) {
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}

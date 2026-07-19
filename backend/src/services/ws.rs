use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

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
    pub is_ai: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OnlineUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: String,
    pub watching: Option<String>,
}

#[derive(Default)]
pub struct PresenceRegistry {
    pub connections: HashMap<u64, OnlineUser>,
    pub next_id: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WsFighter {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
}

// ── WsEvent ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    NewComment(WsComment),
    UpdateComment(WsComment),
    DeleteComment {
        id: i32,
        video_id: String,
    },
    UpdateBout(WsBout),
    UpdateVideoScore {
        video_id: String,
        total_score_a: i32,
        total_score_b: i32,
    },
    UpdateVideoFighters {
        video_id: String,
        fighter_a: Option<WsFighter>,
        fighter_b: Option<WsFighter>,
    },
    NewVideo {
        id: String,
        date: String,
        preview_url: String,
    },
    VideoRemoved {
        id: String,
    },
    PresenceUpdate {
        users: Vec<OnlineUser>,
    },
    UpdateVideoAiLabeled {
        video_id: String,
        is_ai_labeled: bool,
        is_analyzing: bool,
    },
}

impl WsEvent {
    pub fn is_gallery_event(&self) -> bool {
        matches!(
            self,
            WsEvent::UpdateVideoAiLabeled { .. }
                | WsEvent::UpdateVideoScore { .. }
                | WsEvent::UpdateVideoFighters { .. }
                | WsEvent::NewVideo { .. }
                | WsEvent::VideoRemoved { .. }
                | WsEvent::PresenceUpdate { .. }
        )
    }

    pub fn video_id(&self) -> Option<&str> {
        match self {
            WsEvent::NewComment(c) => Some(&c.video_id),
            WsEvent::UpdateComment(c) => Some(&c.video_id),
            WsEvent::DeleteComment { video_id, .. } => Some(video_id),
            WsEvent::UpdateBout(b) => Some(&b.video_id),
            WsEvent::UpdateVideoScore { video_id, .. } => Some(video_id),
            WsEvent::UpdateVideoFighters { video_id, .. } => Some(video_id),
            WsEvent::UpdateVideoAiLabeled { video_id, .. } => Some(video_id),
            WsEvent::NewVideo { .. } => None,
            WsEvent::VideoRemoved { .. } => None,
            WsEvent::PresenceUpdate { .. } => None,
        }
    }
}

pub type WsHub = broadcast::Sender<WsEvent>;

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn broadcast_presence(registry: &RwLock<PresenceRegistry>, ws_hub: &WsHub) {
    let users: Vec<OnlineUser> = {
        let reg = registry.read().await;
        reg.connections.values().cloned().collect()
    };
    let event = WsEvent::PresenceUpdate { users };
    let _ = ws_hub.send(event);
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // First message must be {"token": "..."}
    let (user_id, is_guest) = match socket.recv().await {
        Some(Ok(Message::Text(text))) => {
            println!("WS [init] received text: {}", text);
            let val: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => return,
            };
            let token = match val.get("token").and_then(|t| t.as_str()) {
                Some(t) => t.to_string(),
                None => return,
            };
            match crate::api::auth::verify_token(&token, &state.jwt_secret) {
                Ok(claims) => (claims.sub, false),
                Err(_) => {
                    // Fallback to ShareClaims token for guest connections
                    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
                    match decode::<crate::api::auth::ShareClaims>(
                        &token,
                        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    ) {
                        Ok(_) => ("guest".to_string(), true),
                        Err(_) => return,
                    }
                }
            }
        }
        _ => return,
    };

    let conn_id = {
        let mut reg = state.presence.write().await;
        let id = reg.next_id;
        reg.next_id += 1;
        id
    };

    let online_user = if is_guest {
        let guest_uid = format!("guest_{}", conn_id);
        OnlineUser {
            id: guest_uid.clone(),
            username: guest_uid.clone(),
            display_name: format!("Гость #{}", conn_id),
            avatar_url: "/api/users/guest/avatar".to_string(),
            color: crate::api::auth::generate_color(&guest_uid),
            watching: None,
        }
    } else {
        let db = state.db.clone();
        let uid_clone = user_id.clone();
        let db_user = match tokio::task::spawn_blocking(move || {
            use crate::db::schema::users;
            use diesel::prelude::*;
            let mut conn = db.get().map_err(|e| e.to_string())?;
            users::table
                .filter(users::id.eq(&uid_clone))
                .first::<crate::db::models::User>(&mut conn)
                .map_err(|e| e.to_string())
        })
        .await {
            Ok(Ok(u)) => u,
            _ => return,
        };

        let avatar_url = format!("/api/users/{}/avatar", db_user.id);
        let color = db_user.color.clone().unwrap_or_else(|| crate::api::auth::generate_color(&db_user.id));

        OnlineUser {
            id: db_user.id,
            username: db_user.username,
            display_name: db_user.display_name,
            avatar_url,
            color,
            watching: None,
        }
    };

    println!("WS user connected: {} (conn_id: {})", online_user.display_name, conn_id);

    let current_online_user_id = online_user.id.clone();
    {
        let mut reg = state.presence.write().await;
        reg.connections.insert(conn_id, online_user);
    }

    // Send init event to client so it knows its socket's ID
    let init_msg = serde_json::json!({ "type": "init", "conn_id": conn_id, "user_id": current_online_user_id });
    let _ = socket.send(Message::Text(init_msg.to_string().into())).await;

    let mut rx = state.ws_hub.subscribe();

    // Broadcast updated presence list
    broadcast_presence(&state.presence, &state.ws_hub).await;

    let mut watching: Option<String> = None;

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        println!("WS conn_id {} received text: {}", conn_id, text);
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(watching_val) = val.get("watching") {
                                let next_watching = watching_val.as_str().map(|s| s.to_string());
                                println!("WS conn_id {} updated watching to: {:?}", conn_id, next_watching);
                                watching = next_watching.clone();

                                // Update registry
                                {
                                    let mut reg = state.presence.write().await;
                                    if let Some(user) = reg.connections.get_mut(&conn_id) {
                                        user.watching = next_watching;
                                    }
                                }
                                broadcast_presence(&state.presence, &state.ws_hub).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        println!("WS conn_id {} closed", conn_id);
                        break;
                    }
                    _ => {}
                }
            }
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        let should_send = if ev.is_gallery_event() {
                            true
                        } else {
                            match ev.video_id() {
                                Some(vid) => watching.as_deref() == Some(vid),
                                None => true,
                            }
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

    println!("WS conn_id {} cleaning up from registry", conn_id);
    // Clean up registry on disconnect
    {
        let mut reg = state.presence.write().await;
        reg.connections.remove(&conn_id);
    }
    broadcast_presence(&state.presence, &state.ws_hub).await;
}

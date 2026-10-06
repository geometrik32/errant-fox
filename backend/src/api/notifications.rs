use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        models::{Notification, User, Video},
        schema::{notifications, users, videos},
    },
    errors::AppError,
    middleware::auth::CurrentUser,
    services::ws::WsEvent,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSummaryDto {
    pub id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationDto {
    pub id: i32,
    pub user_id: String,
    pub actor: UserSummaryDto,
    pub event_type: String,
    pub video_id: String,
    pub video_title: String,
    pub bout_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub payload: Option<serde_json::Value>,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub is_read: Option<bool>,
    pub event_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationsResponse {
    pub items: Vec<NotificationDto>,
    pub total: i64,
    pub unread_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReadByVideoRequest {
    pub video_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ReadBoutRequest {
    pub bout_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReadCommentRequest {
    pub comment_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReadMultipleRequest {
    pub ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub unread_count: i64,
}

/// Helper to get formatted video title
pub fn get_video_title_db(v: &Video, conn: &mut diesel::SqliteConnection) -> String {
    let date_str = v.date.format("%d.%m.%Y").to_string();
    let name_a = v.fighter_a_id.as_ref().and_then(|id_a| {
        users::table
            .filter(users::id.eq(id_a))
            .first::<User>(conn)
            .ok()
            .map(|u| u.display_name)
    });
    let name_b = v.fighter_b_id.as_ref().and_then(|id_b| {
        users::table
            .filter(users::id.eq(id_b))
            .first::<User>(conn)
            .ok()
            .map(|u| u.display_name)
    });
    match (name_a, name_b) {
        (Some(a), Some(b)) => format!("{} vs {} ({})", a, b, date_str),
        (Some(a), None) => format!("{} ({})", a, date_str),
        (None, Some(b)) => format!("{} ({})", b, date_str),
        (None, None) => format!("Видео ({})", date_str),
    }
}

/// Helper to calculate unread count for a user
pub fn count_unread(conn: &mut diesel::SqliteConnection, uid: &str) -> Result<i64, diesel::result::Error> {
    notifications::table
        .filter(notifications::user_id.eq(uid))
        .filter(notifications::is_read.eq(false))
        .count()
        .get_result(conn)
}

/// Broadcast updated unread count over WebSocket
pub fn broadcast_unread_count(state: &AppState, user_id: &str, unread_count: i64) {
    let _ = state.ws_hub.send(WsEvent::UpdateNotificationUnreadCount {
        user_id: user_id.to_string(),
        unread_count,
    });
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_notifications(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(q): Query<ListNotificationsQuery>,
) -> Result<Json<NotificationsResponse>, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();
    let limit = q.limit.unwrap_or(100).min(200);
    let offset = q.offset.unwrap_or(0);
    let filter_read = q.is_read;
    let filter_event_type = q.event_type;

    let res = tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut query = notifications::table
            .filter(notifications::user_id.eq(&uid))
            .into_boxed();

        if let Some(is_read) = filter_read {
            query = query.filter(notifications::is_read.eq(is_read));
        }

        if let Some(ref et) = filter_event_type {
            query = query.filter(notifications::event_type.eq(et));
        }

        let total: i64 = query
            .select(diesel::dsl::count(notifications::id))
            .get_result(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let unread_count = count_unread(&mut conn, &uid)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Reconstruct query for items
        let mut query = notifications::table
            .filter(notifications::user_id.eq(&uid))
            .into_boxed();

        if let Some(is_read) = filter_read {
            query = query.filter(notifications::is_read.eq(is_read));
        }
        if let Some(ref et) = filter_event_type {
            query = query.filter(notifications::event_type.eq(et));
        }

        let raw_notifications = query
            .order(notifications::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<Notification>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Pre-fetch actors and videos
        let actor_ids: Vec<String> = raw_notifications.iter().map(|n| n.actor_id.clone()).collect();
        let video_ids: Vec<String> = raw_notifications.iter().map(|n| n.video_id.clone()).collect();

        let actors_map: std::collections::HashMap<String, UserSummaryDto> = users::table
            .filter(users::id.eq_any(&actor_ids))
            .load::<User>(&mut conn)
            .unwrap_or_default()
            .into_iter()
            .map(|u| {
                let avatar = format!("/api/users/{}/avatar", u.id);
                (
                    u.id.clone(),
                    UserSummaryDto {
                        id: u.id,
                        display_name: u.display_name,
                        avatar_url: Some(avatar),
                        color: u.color,
                    },
                )
            })
            .collect();

        let videos_map: std::collections::HashMap<String, String> = videos::table
            .filter(videos::id.eq_any(&video_ids))
            .load::<Video>(&mut conn)
            .unwrap_or_default()
            .into_iter()
            .map(|v| {
                let title = get_video_title_db(&v, &mut conn);
                (v.id, title)
            })
            .collect();

        let items: Vec<NotificationDto> = raw_notifications
            .into_iter()
            .map(|n| {
                let actor = actors_map.get(&n.actor_id).cloned().unwrap_or_else(|| UserSummaryDto {
                    id: n.actor_id.clone(),
                    display_name: "Пользователь".to_string(),
                    avatar_url: Some(format!("/api/users/{}/avatar", n.actor_id)),
                    color: None,
                });
                let video_title = videos_map.get(&n.video_id).cloned().unwrap_or_else(|| "Видео".to_string());
                let payload_json = n.payload.as_deref().and_then(|p| serde_json::from_str(p).ok());

                NotificationDto {
                    id: n.id,
                    user_id: n.user_id,
                    actor,
                    event_type: n.event_type,
                    video_id: n.video_id,
                    video_title,
                    bout_id: n.bout_id,
                    comment_id: n.comment_id,
                    payload: payload_json,
                    is_read: n.is_read,
                    created_at: n.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                }
            })
            .collect();

        Ok::<_, AppError>(NotificationsResponse {
            items,
            total,
            unread_count,
        })
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(res))
}

pub async fn get_unread_count(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<UnreadCountResponse>, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();

    let unread_count = tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        count_unread(&mut conn, &uid).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(UnreadCountResponse { unread_count }))
}

pub async fn read_by_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<ReadByVideoRequest>,
) -> Result<StatusCode, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();
    let vid = body.video_id;

    let unread_count = tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        // Mark video_tag, video_labeled, and any general video notifications as read
        diesel::update(
            notifications::table
                .filter(notifications::user_id.eq(&uid))
                .filter(notifications::video_id.eq(&vid))
                .filter(notifications::event_type.eq_any(["video_tag", "video_labeled"]))
                .filter(notifications::is_read.eq(false)),
        )
        .set(notifications::is_read.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        count_unread(&mut conn, &uid).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    broadcast_unread_count(&state, &user.id, unread_count);
    Ok(StatusCode::OK)
}

pub async fn read_bout(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<ReadBoutRequest>,
) -> Result<StatusCode, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();
    let bid = body.bout_id;

    let unread_count = tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(
            notifications::table
                .filter(notifications::user_id.eq(&uid))
                .filter(notifications::bout_id.eq(bid))
                .filter(notifications::is_read.eq(false)),
        )
        .set(notifications::is_read.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        count_unread(&mut conn, &uid).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    broadcast_unread_count(&state, &user.id, unread_count);
    Ok(StatusCode::OK)
}

pub async fn read_comment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<ReadCommentRequest>,
) -> Result<StatusCode, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();
    let cid = body.comment_id;

    let unread_count = tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(
            notifications::table
                .filter(notifications::user_id.eq(&uid))
                .filter(notifications::comment_id.eq(cid))
                .filter(notifications::is_read.eq(false)),
        )
        .set(notifications::is_read.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        count_unread(&mut conn, &uid).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    broadcast_unread_count(&state, &user.id, unread_count);
    Ok(StatusCode::OK)
}

pub async fn read_all(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<StatusCode, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(
            notifications::table
                .filter(notifications::user_id.eq(&uid))
                .filter(notifications::is_read.eq(false)),
        )
        .set(notifications::is_read.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<_, AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    broadcast_unread_count(&state, &user.id, 0);
    Ok(StatusCode::OK)
}

pub async fn read_multiple(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<ReadMultipleRequest>,
) -> Result<StatusCode, AppError> {
    let db = state.db.clone();
    let uid = user.id.clone();
    let ids = body.ids;

    if ids.is_empty() {
        return Ok(StatusCode::OK);
    }

    let unread_count = tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(
            notifications::table
                .filter(notifications::user_id.eq(&uid))
                .filter(notifications::id.eq_any(&ids))
                .filter(notifications::is_read.eq(false)),
        )
        .set(notifications::is_read.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        count_unread(&mut conn, &uid).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    broadcast_unread_count(&state, &user.id, unread_count);
    Ok(StatusCode::OK)
}

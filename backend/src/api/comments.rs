use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::{Comment, CommentReaction, NewComment, User},
    errors::AppError,
    middleware::auth::CurrentUser,
    state::AppState,
    ws::{WsComment, WsCommentAuthor, WsEvent},
};

// ── Response DTO ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CommentAuthorResponse {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
}

#[derive(Serialize)]
pub struct CommentResponse {
    pub id: i32,
    pub author: CommentAuthorResponse,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<String>,
    pub likes: i32,
    pub dislikes: i32,
    pub my_reaction: Option<String>,
}

fn to_response(c: &Comment, author: &User, likes: i32, dislikes: i32, my_reaction: Option<String>) -> CommentResponse {
    CommentResponse {
        id: c.id,
        author: CommentAuthorResponse {
            id: author.id.clone(),
            display_name: author.display_name.clone(),
            avatar_url: format!("/api/users/{}/avatar", author.id),
        },
        timestamp_ms: c.timestamp_ms,
        text: c.text.clone(),
        reply_to_id: c.reply_to_id,
        created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        edited_at: c
            .edited_at
            .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        likes,
        dislikes,
        my_reaction,
    }
}

fn load_reactions(comment_id: i32, user_id: &str, conn: &mut diesel::SqliteConnection) -> Result<(i32, i32, Option<String>), AppError> {
    use crate::db::schema::comment_reactions;
    let reactions = comment_reactions::table
        .filter(comment_reactions::comment_id.eq(comment_id))
        .load::<CommentReaction>(conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut likes = 0i32;
    let mut dislikes = 0i32;
    let mut my_reaction = None;
    for r in &reactions {
        if r.kind == "like" { likes += 1; } else { dislikes += 1; }
        if r.user_id == user_id { my_reaction = Some(r.kind.clone()); }
    }
    Ok((likes, dislikes, my_reaction))
}

fn to_ws_comment(c: &Comment, author: &User) -> WsComment {
    WsComment {
        id: c.id,
        video_id: c.video_id.clone(),
        author: WsCommentAuthor {
            id: author.id.clone(),
            display_name: author.display_name.clone(),
            avatar_url: format!("/api/users/{}/avatar", author.id),
        },
        timestamp_ms: c.timestamp_ms,
        text: c.text.clone(),
        reply_to_id: c.reply_to_id,
        created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        edited_at: None,
    }
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub video_id: String,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct PatchCommentRequest {
    pub text: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn post_comment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

    let comment = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{comments, videos};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            videos::table.filter(videos::id.eq(&body.video_id)),
        ))
        .get_result(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if !exists {
            return Err(AppError::NotFound);
        }

        diesel::insert_into(comments::table)
            .values(&NewComment {
                video_id: body.video_id.clone(),
                author_id: user_id.clone(),
                timestamp_ms: body.timestamp_ms,
                text: body.text.clone(),
                reply_to_id: body.reply_to_id,
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        comments::table
            .filter(comments::author_id.eq(&user_id))
            .order(comments::id.desc())
            .first::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state
        .ws_hub
        .send(WsEvent::NewComment(to_ws_comment(&comment, &user)));

    Ok((StatusCode::CREATED, Json(to_response(&comment, &user, 0, 0, None))))
}

pub async fn patch_comment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i32>,
    Json(body): Json<PatchCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();
    let new_text = body.text.clone();

    let (comment, likes, dislikes, my_reaction) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::comments;

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let cur = comments::table
            .filter(comments::id.eq(id))
            .first::<Comment>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        if cur.author_id != user_id {
            return Err(AppError::Forbidden);
        }

        let now = Utc::now().naive_utc();

        diesel::update(comments::table.filter(comments::id.eq(id)))
            .set((
                comments::text.eq(&new_text),
                comments::edited_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let updated = comments::table
            .filter(comments::id.eq(id))
            .first::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let (likes, dislikes, my_reaction) = load_reactions(id, &user_id, &mut conn)?;
        Ok::<_, AppError>((updated, likes, dislikes, my_reaction))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(to_response(&comment, &user, likes, dislikes, my_reaction)))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();
    let is_admin = user.is_admin;

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::comments;

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let cur = comments::table
            .filter(comments::id.eq(id))
            .first::<Comment>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        if cur.author_id != user_id && !is_admin {
            return Err(AppError::Forbidden);
        }

        diesel::delete(comments::table.filter(comments::id.eq(id)))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── Reactions ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ReactRequest {
    pub kind: String,
}

pub async fn react_comment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i32>,
    Json(body): Json<ReactRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.kind != "like" && body.kind != "dislike" {
        return Err(AppError::BadRequest("kind must be 'like' or 'dislike'".to_string()));
    }
    let db = state.db.clone();
    let user_id = user.id.clone();
    let kind = body.kind.clone();

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::{comment_reactions, comments};
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            comments::table.filter(comments::id.eq(id)),
        ))
        .get_result(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;
        if !exists {
            return Err(AppError::NotFound);
        }

        diesel::replace_into(comment_reactions::table)
            .values(&CommentReaction { comment_id: id, user_id, kind })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn delete_react(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::comment_reactions;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        diesel::delete(
            comment_reactions::table
                .filter(comment_reactions::comment_id.eq(id))
                .filter(comment_reactions::user_id.eq(&user_id)),
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "ok": true })))
}

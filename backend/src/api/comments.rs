use axum::{
    extract::{Path, Query, State},
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
    services::ws::{WsComment, WsCommentAuthor, WsEvent},
};

// ── Response DTO ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CommentAuthorResponse {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
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
    pub bout_id: Option<i32>,
}

fn to_response(c: &Comment, author: &User, likes: i32, dislikes: i32, my_reaction: Option<String>) -> CommentResponse {
    let (display_name, color) = if let Some(ref nick) = c.guest_nickname {
        (nick.clone(), Some(crate::api::auth::generate_color(nick)))
    } else {
        (author.display_name.clone(), author.color.clone())
    };

    CommentResponse {
        id: c.id,
        author: CommentAuthorResponse {
            id: author.id.clone(),
            display_name,
            avatar_url: format!("/api/users/{}/avatar", author.id),
            color,
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
        bout_id: c.bout_id,
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
    let (display_name, color) = if let Some(ref nick) = c.guest_nickname {
        (nick.clone(), Some(crate::api::auth::generate_color(nick)))
    } else {
        (author.display_name.clone(), author.color.clone())
    };

    WsComment {
        id: c.id,
        video_id: c.video_id.clone(),
        author: WsCommentAuthor {
            id: author.id.clone(),
            display_name,
            avatar_url: format!("/api/users/{}/avatar", author.id),
            color,
        },
        timestamp_ms: c.timestamp_ms,
        text: c.text.clone(),
        reply_to_id: c.reply_to_id,
        created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        edited_at: None,
        bout_id: c.bout_id,
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

struct NotificationTarget {
    vk_id: String,
    message: String,
}

pub async fn post_comment(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    if user.role == "guest" {
        return Err(AppError::Forbidden);
    }
    let db = state.db.clone();
    let user_id = user.id.clone();
    let frontend_origin = state.frontend_url.clone();
    let commenter_name = user.display_name.clone();

    let (comment, notifications) = tokio::task::spawn_blocking(move || {
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

        // Auto-detect which bout (if any) contains this timestamp
        let bout_id: Option<i32> = {
            use crate::db::schema::bouts;
            bouts::table
                .filter(bouts::video_id.eq(&body.video_id))
                .filter(bouts::time_start_ms.le(body.timestamp_ms))
                .filter(bouts::time_end_ms.ge(body.timestamp_ms))
                .select(bouts::id)
                .first::<i32>(&mut conn)
                .optional()
                .unwrap_or(None)
        };

        diesel::insert_into(comments::table)
            .values(&NewComment {
                video_id: body.video_id.clone(),
                author_id: user_id.clone(),
                timestamp_ms: body.timestamp_ms,
                text: body.text.clone(),
                reply_to_id: body.reply_to_id,
                bout_id,
                guest_nickname: None,
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let comment = comments::table
            .filter(comments::author_id.eq(&user_id))
            .order(comments::id.desc())
            .first::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut notifications = Vec::new();

        let video = videos::table
            .filter(videos::id.eq(&comment.video_id))
            .first::<crate::db::models::Video>(&mut conn)
            .optional()
            .unwrap_or(None);

        if let Some(video) = video {
            let get_display_name = |v: &crate::db::models::Video, c: &mut diesel::SqliteConnection| -> String {
                use crate::db::schema::users;
                let date_str = v.date.format("%d.%m.%Y").to_string();
                
                let name_a = v.fighter_a_id.as_ref().and_then(|id_a| {
                    users::table.filter(users::id.eq(id_a))
                        .first::<crate::db::models::User>(c)
                        .ok()
                        .map(|u| u.display_name)
                });
                
                let name_b = v.fighter_b_id.as_ref().and_then(|id_b| {
                    users::table.filter(users::id.eq(id_b))
                        .first::<crate::db::models::User>(c)
                        .ok()
                        .map(|u| u.display_name)
                });
                
                match (name_a, name_b) {
                    (Some(a), Some(b)) => format!("{} vs {} ({})", a, b, date_str),
                    (Some(a), None) => format!("{} ({})", a, date_str),
                    (None, Some(b)) => format!("{} ({})", b, date_str),
                    (None, None) => format!("Без названия ({})", date_str),
                }
            };

            let video_title = get_display_name(&video, &mut conn);
            let mut notified_reply_user_id: Option<String> = None;

            // A. Check if it's a reply
            if let Some(parent_id) = comment.reply_to_id {
                if let Ok(parent_comment) = comments::table.filter(comments::id.eq(parent_id)).first::<Comment>(&mut conn) {
                    if parent_comment.author_id != comment.author_id {
                        use crate::db::schema::users;
                        if let Ok(parent_author) = users::table.filter(users::id.eq(&parent_comment.author_id)).first::<User>(&mut conn) {
                            if let Some(ref vk_id_str) = parent_author.vk_id {
                                if !vk_id_str.trim().is_empty() {
                                    let msg = format!(
                                        "💬 Пользователь {} ответил на ваш комментарий к бою {}:\n\"{}\"\n\nСсылка: {}/#/player/{}?t={}",
                                        commenter_name,
                                        video_title,
                                        comment.text,
                                        frontend_origin,
                                        comment.video_id,
                                        comment.timestamp_ms
                                    );
                                    notifications.push(NotificationTarget {
                                        vk_id: vk_id_str.clone(),
                                        message: msg,
                                    });
                                    notified_reply_user_id = Some(parent_comment.author_id.clone());
                                }
                            }
                        }
                    }
                }
            }

            // B. Notify participants
            let mut participants = Vec::new();
            if let Some(ref fid) = video.fighter_a_id {
                participants.push(fid.clone());
            }
            if let Some(ref fid) = video.fighter_b_id {
                participants.push(fid.clone());
            }

            for part_id in participants {
                if part_id != comment.author_id && Some(&part_id) != notified_reply_user_id.as_ref() {
                    use crate::db::schema::users;
                    if let Ok(part_user) = users::table.filter(users::id.eq(&part_id)).first::<User>(&mut conn) {
                        if let Some(ref vk_id_str) = part_user.vk_id {
                            if !vk_id_str.trim().is_empty() {
                                let msg = format!(
                                    "💬 В вашем бою {} пользователь {} оставил новый комментарий:\n\"{}\"\n\nСсылка: {}/#/player/{}?t={}",
                                    video_title,
                                    commenter_name,
                                    comment.text,
                                    frontend_origin,
                                    comment.video_id,
                                    comment.timestamp_ms
                                );
                                notifications.push(NotificationTarget {
                                    vk_id: vk_id_str.clone(),
                                    message: msg,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok::<_, AppError>((comment, notifications))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state
        .ws_hub
        .send(WsEvent::NewComment(to_ws_comment(&comment, &user)));

    // Spawn VK notification tasks
    let vk_notifier = state.vk_notifier.clone();
    for target in notifications {
        let notifier = vk_notifier.clone();
        tokio::spawn(async move {
            notifier.send_notification(&target.vk_id, &target.message).await;
        });
    }

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

    let _ = state.ws_hub.send(WsEvent::UpdateComment(to_ws_comment(&comment, &user)));

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

    let video_id = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{comment_reactions, comments};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let cur = comments::table
            .filter(comments::id.eq(id))
            .first::<Comment>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        if cur.author_id != user_id && !is_admin && cur.author_id != "guest" {
            return Err(AppError::Forbidden);
        }

        let video_id = cur.video_id.clone();

        // Collect IDs of all replies to this comment
        let reply_ids: Vec<i32> = comments::table
            .filter(comments::reply_to_id.eq(id))
            .select(comments::id)
            .load::<i32>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Delete reactions for replies
        if !reply_ids.is_empty() {
            diesel::delete(
                comment_reactions::table.filter(comment_reactions::comment_id.eq_any(&reply_ids)),
            )
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        // Delete reply comments
        diesel::delete(comments::table.filter(comments::reply_to_id.eq(id)))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Delete reactions for the original comment
        diesel::delete(
            comment_reactions::table.filter(comment_reactions::comment_id.eq(id)),
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // Delete the original comment
        diesel::delete(comments::table.filter(comments::id.eq(id)))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok::<String, AppError>(video_id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(WsEvent::DeleteComment {
        id,
        video_id,
    });

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

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub comment_id: i32,
    pub comment_text: String,
    pub author_id: String,
    pub author_name: String,
    pub timestamp_ms: i32,
    pub video_id: String,
    pub video_date: String,
    pub fighter_a_name: Option<String>,
    pub fighter_b_name: Option<String>,
    pub bout_id: Option<i32>,
    pub bout_order_index: Option<i32>,
}

pub async fn search_comments(
    State(state): State<AppState>,
    CurrentUser(_user): CurrentUser,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let db = state.db.clone();

    let results = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comments, users, videos};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        // Load all comments with author and video info to filter in memory (for unicode case-insensitivity)
        let rows = comments::table
            .inner_join(users::table.on(comments::author_id.eq(users::id)))
            .inner_join(videos::table.on(comments::video_id.eq(videos::id)))
            .select((
                comments::id,
                comments::text,
                comments::timestamp_ms,
                comments::video_id,
                comments::bout_id,
                users::id,
                users::display_name,
                videos::date,
                videos::fighter_a_id,
                videos::fighter_b_id,
            ))
            .load::<(i32, String, i32, String, Option<i32>, String, String, chrono::NaiveDate, Option<String>, Option<String>)>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let query_lower = params.q.to_lowercase();
        let mut out = Vec::new();
        for (cid, ctext, ts, vid, bout_id, uid, uname, vdate, fa_id, fb_id) in rows {
            if ctext.to_lowercase().contains(&query_lower) {
                // Resolve fighter display names
                let fa_name = if let Some(ref id) = fa_id {
                    users::table.filter(users::id.eq(id)).select(users::display_name).first::<String>(&mut conn).ok()
                } else { None };
                let fb_name = if let Some(ref id) = fb_id {
                    users::table.filter(users::id.eq(id)).select(users::display_name).first::<String>(&mut conn).ok()
                } else { None };

                let bout_order = if let Some(bid) = bout_id {
                    bouts::table.filter(bouts::id.eq(bid)).select(bouts::order_index).first::<i32>(&mut conn).ok()
                } else { None };

                out.push(SearchResult {
                    comment_id: cid,
                    comment_text: ctext,
                    author_id: uid,
                    author_name: uname,
                    timestamp_ms: ts,
                    video_id: vid,
                    video_date: vdate.to_string(),
                    fighter_a_name: fa_name,
                    fighter_b_name: fb_name,
                    bout_id,
                    bout_order_index: bout_order,
                });

                if out.len() >= 50 {
                    break;
                }
            }
        }

        Ok::<_, AppError>(out)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(results))
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

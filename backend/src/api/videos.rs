use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    db::models::{Bout, Comment, CommentReaction, User, Video},
    errors::AppError,
    middleware::auth::CurrentUser,
    state::AppState,
    services::ws::{WsEvent, WsFighter},
};

fn is_not_found_error(err: &anyhow::Error) -> bool {
    if let Some(reqwest_err) = err.downcast_ref::<reqwest::Error>() {
        if let Some(status) = reqwest_err.status() {
            return status == reqwest::StatusCode::NOT_FOUND;
        }
    }
    let err_str = err.to_string().to_lowercase();
    err_str.contains("404") || err_str.contains("not found")
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VideoFighterDto {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct VideoListDto {
    pub id: String,
    pub date: String,
    pub fighter_a: Option<VideoFighterDto>,
    pub fighter_b: Option<VideoFighterDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_score_a: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_score_b: Option<i32>,
    pub is_tagged: bool,
    pub preview_url: String,
    pub preview_count: i32,
    pub is_ai_labeled: bool,
    pub is_analyzing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seafile_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seafile_web_url: Option<String>,
}

#[derive(Serialize)]
pub struct BoutDto {
    pub id: i32,
    pub order_index: i32,
    pub time_start_ms: i32,
    pub time_end_ms: i32,
    pub score_a: i32,
    pub score_b: i32,
    pub technique_a_id: Option<i32>,
    pub hit_zone_a: Option<String>,
    pub result_a: Option<String>,
    pub technique_b_id: Option<i32>,
    pub hit_zone_b: Option<String>,
    pub result_b: Option<String>,
    pub is_ai: bool,
}

#[derive(Serialize)]
pub struct CommentAuthorDto {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct CommentDto {
    pub id: i32,
    pub author: CommentAuthorDto,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub created_at: String,
    pub likes: i32,
    pub dislikes: i32,
    pub my_reaction: Option<String>,
}

#[derive(Serialize)]
pub struct VideoFullDto {
    pub id: String,
    pub date: String,
    pub fighter_a: Option<VideoFighterDto>,
    pub fighter_b: Option<VideoFighterDto>,
    pub stream_url: String,
    pub duration_ms: Option<i32>,
    pub fps: Option<f32>,
    pub is_ai_labeled: bool,
    pub is_analyzing: bool,
    pub bouts: Vec<BoutDto>,
    pub comments: Vec<CommentDto>,
}

// ── Query params / request bodies ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VideoListQuery {
    pub fighter_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Deserialize)]
pub struct PatchVideoRequest {
    pub fighter_a_id: Option<String>,
    pub fighter_b_id: Option<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn fighter_dto(u: &User) -> VideoFighterDto {
    VideoFighterDto {
        id: u.id.clone(),
        display_name: u.display_name.clone(),
        avatar_url: format!("/api/users/{}/avatar", u.id),
        color: u.color.clone(),
    }
}

fn bout_dto(b: &Bout) -> BoutDto {
    BoutDto {
        id: b.id,
        order_index: b.order_index,
        time_start_ms: b.time_start_ms,
        time_end_ms: b.time_end_ms,
        score_a: b.score_a,
        score_b: b.score_b,
        technique_a_id: b.technique_a_id,
        hit_zone_a: b.hit_zone_a.clone(),
        result_a: b.result_a.clone(),
        technique_b_id: b.technique_b_id,
        hit_zone_b: b.hit_zone_b.clone(),
        result_b: b.result_b.clone(),
        is_ai: b.is_ai,
    }
}

fn build_video_full(
    video: &Video,
    bouts: Vec<Bout>,
    comments: Vec<Comment>,
    users_map: &HashMap<String, User>,
    reactions_map: &HashMap<i32, (i32, i32, Option<String>)>,
    stream_url: String,
) -> VideoFullDto {
    let fighter_a = video
        .fighter_a_id
        .as_ref()
        .and_then(|id| users_map.get(id))
        .map(fighter_dto);
    let fighter_b = video
        .fighter_b_id
        .as_ref()
        .and_then(|id| users_map.get(id))
        .map(fighter_dto);

    let comment_dtos = comments
        .iter()
        .map(|c| {
            let mut author = users_map
                .get(&c.author_id)
                .map(|u| CommentAuthorDto {
                    id: u.id.clone(),
                    display_name: u.display_name.clone(),
                    avatar_url: format!("/api/users/{}/avatar", u.id),
                    color: u.color.clone(),
                })
                .unwrap_or_else(|| CommentAuthorDto {
                    id: c.author_id.clone(),
                    display_name: "Unknown".to_string(),
                    avatar_url: format!("/api/users/{}/avatar", c.author_id),
                    color: None,
                });
            if let Some(ref nick) = c.guest_nickname {
                author.display_name = nick.clone();
                author.color = Some(crate::api::auth::generate_color(nick));
            }
            let (likes, dislikes, my_reaction) = reactions_map
                .get(&c.id)
                .cloned()
                .unwrap_or((0, 0, None));
            CommentDto {
                id: c.id,
                author,
                timestamp_ms: c.timestamp_ms,
                text: c.text.clone(),
                reply_to_id: c.reply_to_id,
                created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                likes,
                dislikes,
                my_reaction,
            }
        })
        .collect();

    VideoFullDto {
        id: video.id.clone(),
        date: video.date.format("%Y-%m-%d").to_string(),
        fighter_a,
        fighter_b,
        stream_url,
        duration_ms: video.duration_ms,
        fps: video.fps,
        is_ai_labeled: video.is_ai_labeled,
        is_analyzing: video.is_analyzing,
        bouts: bouts.iter().map(bout_dto).collect(),
        comments: comment_dtos,
    }
}

fn load_users_for_video(
    video: &Video,
    comments: &[Comment],
    conn: &mut diesel::SqliteConnection,
) -> Result<HashMap<String, User>, AppError> {
    use crate::db::schema::users;

    let mut ids: Vec<String> = Vec::new();
    if let Some(ref id) = video.fighter_a_id {
        ids.push(id.clone());
    }
    if let Some(ref id) = video.fighter_b_id {
        ids.push(id.clone());
    }
    for c in comments {
        ids.push(c.author_id.clone());
    }
    ids.sort();
    ids.dedup();

    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let user_list = users::table
        .filter(users::id.eq_any(&ids))
        .load::<User>(conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(user_list.into_iter().map(|u| (u.id.clone(), u)).collect())
}

fn build_reactions_map(
    reactions: Vec<CommentReaction>,
    current_user_id: &str,
) -> HashMap<i32, (i32, i32, Option<String>)> {
    let mut map: HashMap<i32, (i32, i32, Option<String>)> = HashMap::new();
    for r in reactions {
        let entry = map.entry(r.comment_id).or_insert((0, 0, None));
        if r.kind == "like" {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
        if r.user_id == current_user_id {
            entry.2 = Some(r.kind.clone());
        }
    }
    map
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_videos(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(params): Query<VideoListQuery>,
) -> Result<Json<Vec<VideoListDto>>, AppError> {
    if _user.0.role == "guest" {
        return Err(AppError::Forbidden);
    }
    let date_from = params
        .date_from
        .as_deref()
        .map(|s| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid date_from".to_string()))
        })
        .transpose()?;
    let date_to = params
        .date_to
        .as_deref()
        .map(|s| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid date_to".to_string()))
        })
        .transpose()?;

    let db = state.db.clone();
    let fighter_id = params.fighter_id.clone();
    let is_admin = _user.0.is_admin;

    let result = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, users, videos};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut query = videos::table.into_boxed();
        if let Some(ref fid) = fighter_id {
            query = query.filter(
                videos::fighter_a_id
                    .eq(fid)
                    .or(videos::fighter_b_id.eq(fid)),
            );
        }
        if let Some(from) = date_from {
            query = query.filter(videos::date.ge(from));
        }
        if let Some(to) = date_to {
            query = query.filter(videos::date.le(to));
        }

        let video_list = query
            .order(videos::date.desc())
            .load::<Video>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if video_list.is_empty() {
            return Ok(vec![]);
        }

        // Collect user IDs from fighters
        let mut user_ids: Vec<String> = Vec::new();
        for v in &video_list {
            if let Some(ref id) = v.fighter_a_id {
                user_ids.push(id.clone());
            }
            if let Some(ref id) = v.fighter_b_id {
                user_ids.push(id.clone());
            }
        }
        user_ids.sort();
        user_ids.dedup();

        let users_map: HashMap<String, User> = if user_ids.is_empty() {
            HashMap::new()
        } else {
            users::table
                .filter(users::id.eq_any(&user_ids))
                .load::<User>(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?
                .into_iter()
                .map(|u| (u.id.clone(), u))
                .collect()
        };

        // Load all bouts for score sums
        let video_ids: Vec<String> = video_list.iter().map(|v| v.id.clone()).collect();
        let all_bouts = bouts::table
            .filter(bouts::video_id.eq_any(&video_ids))
            .load::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut bouts_by_video: HashMap<String, Vec<&Bout>> = HashMap::new();
        for b in &all_bouts {
            bouts_by_video.entry(b.video_id.clone()).or_default().push(b);
        }

        let dtos: Vec<VideoListDto> = video_list
            .iter()
            .map(|v| {
                let fighter_a = v
                    .fighter_a_id
                    .as_ref()
                    .and_then(|id| users_map.get(id))
                    .map(fighter_dto);
                let fighter_b = v
                    .fighter_b_id
                    .as_ref()
                    .and_then(|id| users_map.get(id))
                    .map(fighter_dto);

                let is_tagged = v.fighter_a_id.is_some() && v.fighter_b_id.is_some();

                let (total_score_a, total_score_b) = if is_tagged {
                    let bouts = bouts_by_video
                        .get(&v.id)
                        .map(|b| b.as_slice())
                        .unwrap_or(&[]);
                    let sa: i32 = bouts.iter().map(|b| b.score_a).sum();
                    let sb: i32 = bouts.iter().map(|b| b.score_b).sum();
                    (Some(sa), Some(sb))
                } else {
                    (None, None)
                };

                VideoListDto {
                    id: v.id.clone(),
                    date: v.date.format("%Y-%m-%d").to_string(),
                    fighter_a,
                    fighter_b,
                    total_score_a,
                    total_score_b,
                    is_tagged,
                    preview_url: format!("/api/videos/{}/previews/0", v.id),
                    preview_count: v.preview_count,
                    is_ai_labeled: v.is_ai_labeled,
                    is_analyzing: v.is_analyzing,
                    seafile_path: if is_admin { Some(v.seafile_path.clone()) } else { None },
                    seafile_web_url: if is_admin {
                        Some(format!(
                            "https://seafile.aat-terra.ru/lib/3981eb27-f4c1-4c6d-a05e-5448ee140b8f/file/{}",
                            v.seafile_path
                        ))
                    } else {
                        None
                    },
                }
            })
            .collect();

        Ok(dtos)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn get_video_dto_impl(
    state: &AppState,
    video_id: &str,
    user_id: Option<&str>,
) -> Result<VideoFullDto, AppError> {
    let db = state.db.clone();
    let user_id_opt = user_id.map(|s| s.to_string());

    // 1. Fetch video record first
    let video_id_clone = video_id.to_string();
    let db_clone = db.clone();
    let video = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let video = videos::table
            .filter(videos::id.eq(&video_id_clone))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;
        Ok::<Video, AppError>(video)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // 2. Check physical existence in Seafile "on-the-fly"
    if let Err(e) = state.seafile.get_download_url(&video.seafile_path).await {
        if is_not_found_error(&e) {
            tracing::info!("Video file not found in Seafile. Cascade deleting video_id: {}", video.id);
            let _ = crate::services::sync::delete_videos_cascade(&state.db, &state.previews_dir, &[video.id.clone()]).await;
            let _ = state.ws_hub.send(WsEvent::VideoRemoved { id: video.id.clone() });
            return Err(AppError::NotFound);
        }
    }

    // 3. Load relations and build full DTO
    let video_id_clone = video_id.to_string();
    let db_clone = db.clone();
    let dto = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comment_reactions, comments};

        let mut conn = db_clone.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let video_bouts = bouts::table
            .filter(bouts::video_id.eq(&video_id_clone))
            .order(bouts::order_index.asc())
            .load::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let video_comments = comments::table
            .filter(comments::video_id.eq(&video_id_clone))
            .order(comments::id.asc())
            .load::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let comment_ids: Vec<i32> = video_comments.iter().map(|c| c.id).collect();
        let reactions: Vec<CommentReaction> = if comment_ids.is_empty() {
            vec![]
        } else {
            comment_reactions::table
                .filter(comment_reactions::comment_id.eq_any(&comment_ids))
                .load::<CommentReaction>(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?
        };
        let reactions_map = build_reactions_map(reactions, user_id_opt.as_deref().unwrap_or(""));

        let users_map = load_users_for_video(&video, &video_comments, &mut conn)?;

        Ok::<_, AppError>(build_video_full(
            &video,
            video_bouts,
            video_comments,
            &users_map,
            &reactions_map,
            String::new(),
        ))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let stream_url = format!("/api/videos/{}/stream", dto.id);

    Ok(VideoFullDto { stream_url, ..dto })
}

pub async fn get_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(video_id): Path<String>,
) -> Result<Json<VideoFullDto>, AppError> {
    if user.role == "guest" {
        return Err(AppError::Forbidden);
    }
    let dto = get_video_dto_impl(&state, &video_id, Some(&user.id)).await?;
    Ok(Json(dto))
}

#[derive(Deserialize)]
pub struct SharedVideoQuery {
    pub token: String,
}

pub async fn get_shared_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Query(query): Query<SharedVideoQuery>,
) -> Result<Json<VideoFullDto>, AppError> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    let claims: crate::api::auth::ShareClaims = decode::<crate::api::auth::ShareClaims>(
        &query.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid share token: {}", e)))?;

    if claims.video_id != video_id {
        return Err(AppError::Unauthorized("Invalid share token for this video".to_string()));
    }

    let dto = get_video_dto_impl(&state, &video_id, None).await?;
    Ok(Json(dto))
}

#[derive(Deserialize)]
pub struct CreateShareRequest {
    pub bout_id: Option<i32>,
}

#[derive(Serialize)]
pub struct CreateShareResponse {
    pub token: String,
}

pub async fn create_share_token(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(video_id): Path<String>,
    Json(body): Json<CreateShareRequest>,
) -> Result<Json<CreateShareResponse>, AppError> {
    let token = crate::api::auth::make_share_token(&video_id, body.bout_id, &state.jwt_secret)?;
    Ok(Json(CreateShareResponse { token }))
}

#[derive(Deserialize)]
pub struct CreateSharedCommentRequest {
    pub nickname: String,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub timestamp_ms: i32,
    pub bout_id: Option<i32>,
}

pub async fn create_shared_comment(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Query(query): Query<SharedVideoQuery>,
    Json(body): Json<CreateSharedCommentRequest>,
) -> Result<Json<crate::api::comments::CommentResponse>, AppError> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    let claims: crate::api::auth::ShareClaims = decode::<crate::api::auth::ShareClaims>(
        &query.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid share token: {}", e)))?;

    if claims.video_id != video_id {
        return Err(AppError::Unauthorized("Invalid share token for this video".to_string()));
    }

    let nickname = body.nickname.trim().to_string();
    if nickname.is_empty() {
        return Err(AppError::BadRequest("Nickname cannot be empty".to_string()));
    }

    if body.text.trim().is_empty() {
        return Err(AppError::BadRequest("Comment text cannot be empty".to_string()));
    }

    let guest_id = "guest".to_string();

    let claims_bout_id = claims.bout_id;
    let db_pool = state.db.clone();
    let comment_resp = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users;
        use crate::db::models::{User, NewUser, Comment, NewComment};
        
        let mut conn = db_pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

        // 1. Get or create guest user
        let user = conn.transaction::<User, diesel::result::Error, _>(|tx_conn| {
            let existing = users::table
                .filter(users::id.eq(&guest_id))
                .first::<User>(tx_conn)
                .optional()?;

            if let Some(u) = existing {
                Ok(u)
            } else {
                let new_user = NewUser {
                    id: guest_id.clone(),
                    username: guest_id.clone(),
                    display_name: "Гость".to_string(),
                    password_hash: "guest".to_string(),
                    is_admin: false,
                    avatar_path: None,
                    color: Some("#9E9E9E".to_string()),
                    vk_id: None,
                    role: "guest".to_string(),
                };
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .execute(tx_conn)?;
                
                users::table.filter(users::id.eq(&guest_id)).first::<User>(tx_conn)
            }
        }).map_err(|e| AppError::Internal(e.to_string()))?;

        // 2. Insert new comment
        let new_comment = NewComment {
            video_id: video_id.clone(),
            author_id: user.id.clone(),
            timestamp_ms: body.timestamp_ms,
            text: body.text,
            reply_to_id: body.reply_to_id,
            bout_id: claims_bout_id.or(body.bout_id),
            guest_nickname: Some(nickname.clone()),
        };

        use crate::db::schema::comments;
        let c: Comment = conn.transaction::<Comment, diesel::result::Error, _>(|tx_conn| {
            diesel::insert_into(comments::table)
                .values(&new_comment)
                .execute(tx_conn)?;
            comments::table.order(comments::id.desc()).first::<Comment>(tx_conn)
        }).map_err(|e| AppError::Internal(e.to_string()))?;

        let guest_color = Some(crate::api::auth::generate_color(&nickname));

        // 3. Construct CommentResponse (0 likes, 0 dislikes, no reaction)
        let response = crate::api::comments::CommentResponse {
            id: c.id,
            author: crate::api::comments::CommentAuthorResponse {
                id: user.id.clone(),
                display_name: nickname.clone(),
                avatar_url: format!("/api/users/{}/avatar", user.id),
                color: guest_color.clone(),
            },
            timestamp_ms: c.timestamp_ms,
            text: c.text.clone(),
            reply_to_id: c.reply_to_id,
            created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            edited_at: None,
            likes: 0,
            dislikes: 0,
            my_reaction: None,
            bout_id: c.bout_id,
        };

        // 4. Send WS Event so other watching users see it!
        let ws_event = crate::services::ws::WsEvent::NewComment(crate::services::ws::WsComment {
            id: c.id,
            video_id: c.video_id.clone(),
            author: crate::services::ws::WsCommentAuthor {
                id: user.id.clone(),
                display_name: nickname.clone(),
                avatar_url: format!("/api/users/{}/avatar", user.id),
                color: guest_color,
            },
            timestamp_ms: c.timestamp_ms,
            text: c.text.clone(),
            reply_to_id: c.reply_to_id,
            created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            edited_at: None,
            bout_id: c.bout_id,
        });
        let _ = state.ws_hub.send(ws_event);

        Ok::<_, AppError>(response)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(comment_resp))
}


pub async fn patch_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(video_id): Path<String>,
    Json(body): Json<PatchVideoRequest>,
) -> Result<Json<VideoFullDto>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();
    let frontend_origin = state.frontend_url.clone();
    let video_id_for_db = video_id.clone();

    let (dto, notifications) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comment_reactions, comments, videos, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        // 1. Fetch old video first to see who was already tagged
        let old_video = videos::table
            .filter(videos::id.eq(&video_id_for_db))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let rows = diesel::update(videos::table.filter(videos::id.eq(&video_id_for_db)))
            .set((
                videos::fighter_a_id.eq(&body.fighter_a_id),
                videos::fighter_b_id.eq(&body.fighter_b_id),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound);
        }

        let video = videos::table
            .filter(videos::id.eq(&video_id_for_db))
            .first::<Video>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Collect new tags notifications
        let mut notifications = Vec::new();

        // Check if fighter A is newly added
        if let Some(ref new_a) = video.fighter_a_id {
            if Some(new_a) != old_video.fighter_a_id.as_ref() {
                if let Ok(fighter_user) = users::table.filter(users::id.eq(new_a)).first::<User>(&mut conn) {
                    if let Some(ref vk_id_str) = fighter_user.vk_id {
                        if !vk_id_str.trim().is_empty() {
                            let msg = format!(
                                "⚔️ Вас добавили в качестве участника боя в видео.\n\nСсылка: {}/#/player/{}",
                                frontend_origin,
                                video_id_for_db
                            );
                            notifications.push((vk_id_str.clone(), msg));
                        }
                    }
                }
            }
        }

        // Check if fighter B is newly added
        if let Some(ref new_b) = video.fighter_b_id {
            if Some(new_b) != old_video.fighter_b_id.as_ref() {
                if let Ok(fighter_user) = users::table.filter(users::id.eq(new_b)).first::<User>(&mut conn) {
                    if let Some(ref vk_id_str) = fighter_user.vk_id {
                        if !vk_id_str.trim().is_empty() {
                            let msg = format!(
                                "⚔️ Вас добавили в качестве участника боя в видео.\n\nСсылка: {}/#/player/{}",
                                frontend_origin,
                                video_id_for_db
                            );
                            notifications.push((vk_id_str.clone(), msg));
                        }
                    }
                }
            }
        }

        let video_bouts = bouts::table
            .filter(bouts::video_id.eq(&video_id_for_db))
            .order(bouts::order_index.asc())
            .load::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let video_comments = comments::table
            .filter(comments::video_id.eq(&video_id_for_db))
            .order(comments::id.asc())
            .load::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let comment_ids: Vec<i32> = video_comments.iter().map(|c| c.id).collect();
        let reactions: Vec<CommentReaction> = if comment_ids.is_empty() {
            vec![]
        } else {
            comment_reactions::table
                .filter(comment_reactions::comment_id.eq_any(&comment_ids))
                .load::<CommentReaction>(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?
        };
        let reactions_map = build_reactions_map(reactions, &user_id);

        let users_map = load_users_for_video(&video, &video_comments, &mut conn)?;

        let full_dto = build_video_full(
            &video,
            video_bouts,
            video_comments,
            &users_map,
            &reactions_map,
            String::new(),
        );

        Ok::<_, AppError>((full_dto, notifications))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Spawn async tasks to send notifications
    let vk_notifier = state.vk_notifier.clone();
    for (vk_id, message) in notifications {
        let notifier = vk_notifier.clone();
        tokio::spawn(async move {
            notifier.send_notification(&vk_id, &message).await;
        });
    }

    let stream_url = format!("/api/videos/{}/stream", dto.id);

    // Broadcast update of fighters to WebSocket channel
    let ws_fighter_a = dto.fighter_a.as_ref().map(|f| WsFighter {
        id: f.id.clone(),
        display_name: f.display_name.clone(),
        avatar_url: f.avatar_url.clone(),
        color: f.color.clone(),
    });
    let ws_fighter_b = dto.fighter_b.as_ref().map(|f| WsFighter {
        id: f.id.clone(),
        display_name: f.display_name.clone(),
        avatar_url: f.avatar_url.clone(),
        color: f.color.clone(),
    });
    let _ = state.ws_hub.send(WsEvent::UpdateVideoFighters {
        video_id: video_id.clone(),
        fighter_a: ws_fighter_a,
        fighter_b: ws_fighter_b,
    });

    Ok(Json(VideoFullDto { stream_url, ..dto }))
}

// ── Preview generation ────────────────────────────────────────────────────────

pub async fn get_preview_frame(
    State(state): State<AppState>,
    Path((video_id, frame)): Path<(String, u32)>,
) -> Result<axum::response::Response, AppError> {
    let db = state.db.clone();
    let vid_clone = video_id.clone();

    let (preview_count, seafile_path) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let video = videos::table
            .filter(videos::id.eq(&vid_clone))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;
        Ok::<(i32, String), AppError>((video.preview_count, video.seafile_path))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if preview_count == 0 {
        let seafile = state.seafile.clone();
        let previews_dir = state.previews_dir.clone();
        let db = state.db.clone();
        let vid_id = video_id.clone();
        let server_port = state.server_port;
        let ws_hub = state.ws_hub.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::services::previews::generate_previews(
                &vid_id,
                &seafile,
                &seafile_path,
                std::path::Path::new(&previews_dir),
                &db,
                server_port,
            )
            .await
            {
                tracing::error!("generate_previews failed for {vid_id}: {e:?}");
            } else {
                let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoPreview {
                    video_id: vid_id.clone(),
                    preview_url: format!("/api/videos/{}/previews/0", vid_id),
                });
            }
        });

        return Ok(StatusCode::ACCEPTED.into_response());
    }

    // preview_count == -1 means previous generation attempts failed with
    // permanent HTTP errors (403/404/410) — don't retry.
    if preview_count < 0 {
        return Err(AppError::NotFound);
    }

    let file_path = PathBuf::from(&state.previews_dir)
        .join(&video_id)
        .join(format!("{}.jpg", frame));

    let bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|_| AppError::NotFound)?;

    Ok(([(header::CONTENT_TYPE, "image/jpeg")], bytes).into_response())
}

// ── Video stream proxy ────────────────────────────────────────────────────────

pub async fn stream_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    req_headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let db = state.db.clone();
    let video_id_for_db = video_id.clone();

    let seafile_path = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let video = videos::table
            .filter(videos::id.eq(&video_id_for_db))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;
        Ok::<String, AppError>(video.seafile_path)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let range = req_headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let seafile_resp = match state
        .seafile
        .fetch_range(&seafile_path, range.as_deref())
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            if is_not_found_error(&e) {
                tracing::info!("Video file not found in Seafile. Cascade deleting video_id: {}", video_id);
                let _ = crate::services::sync::delete_videos_cascade(&state.db, &state.previews_dir, &[video_id.clone()]).await;
                let _ = state.ws_hub.send(WsEvent::VideoRemoved { id: video_id.clone() });
                return Err(AppError::NotFound);
            }
            return Err(AppError::Internal(e.to_string()));
        }
    };

    let status = seafile_resp.status();
    let mut builder = axum::response::Response::builder().status(status);

    for key in &["content-type", "content-length", "content-range"] {
        if let Some(val) = seafile_resp.headers().get(*key) {
            builder = builder.header(*key, val);
        }
    }
    builder = builder.header("accept-ranges", "bytes");

    let body = axum::body::Body::from_stream(seafile_resp.bytes_stream());
    builder
        .body(body)
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn download_video_impl(
    state: &AppState,
    video_id: &str,
) -> Result<axum::response::Response, AppError> {
    let db = state.db.clone();
    let vid_clone = video_id.to_string();

    let (video, fighter_a, fighter_b, index) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{videos, users};
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let video = videos::table
            .filter(videos::id.eq(&vid_clone))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let fighter_a = if let Some(ref fid) = video.fighter_a_id {
            users::table
                .filter(users::id.eq(fid))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            None
        };

        let fighter_b = if let Some(ref fid) = video.fighter_b_id {
            users::table
                .filter(users::id.eq(fid))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            None
        };

        // Determine serial number of this video for that day and those fighters
        let same_day_videos = videos::table
            .filter(videos::date.eq(video.date))
            .filter(
                (videos::fighter_a_id.eq(&video.fighter_a_id).and(videos::fighter_b_id.eq(&video.fighter_b_id)))
                .or(videos::fighter_a_id.eq(&video.fighter_b_id).and(videos::fighter_b_id.eq(&video.fighter_a_id)))
            )
            .order(videos::id.asc())
            .load::<Video>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let index = same_day_videos.iter().position(|v| v.id == video.id).unwrap_or(0) + 1;

        Ok::<_, AppError>((video, fighter_a, fighter_b, index))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let filename = {
        let name_a = fighter_a.map(|u| u.display_name).unwrap_or_else(|| "FighterA".to_string());
        let name_b = fighter_b.map(|u| u.display_name).unwrap_or_else(|| "FighterB".to_string());
        let clean_a = crate::api::bouts::transliterate(&name_a);
        let clean_b = crate::api::bouts::transliterate(&name_b);
        let clean_a = if clean_a.is_empty() { "FighterA".to_string() } else { clean_a };
        let clean_b = if clean_b.is_empty() { "FighterB".to_string() } else { clean_b };

        let date_str = video.date.format("%Y-%m-%d").to_string();
        format!("{}_vs_{}_{}_{}.mp4", clean_a, clean_b, date_str, index)
    };

    let seafile_resp = match state
        .seafile
        .fetch_range(&video.seafile_path, None)
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            if is_not_found_error(&e) {
                tracing::info!("Video file not found in Seafile. Cascade deleting video_id: {}", video_id);
                let _ = crate::services::sync::delete_videos_cascade(&state.db, &state.previews_dir, &[video_id.to_string()]).await;
                let _ = state.ws_hub.send(WsEvent::VideoRemoved { id: video_id.to_string() });
                return Err(AppError::NotFound);
            }
            return Err(AppError::Internal(e.to_string()));
        }
    };

    let status = seafile_resp.status();
    let mut builder = axum::response::Response::builder().status(status);

    for key in &["content-type", "content-length"] {
        if let Some(val) = seafile_resp.headers().get(*key) {
            builder = builder.header(*key, val);
        }
    }
    
    builder = builder.header(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename),
    );

    let body = axum::body::Body::from_stream(seafile_resp.bytes_stream());
    builder
        .body(body)
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn download_video(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(video_id): Path<String>,
) -> Result<axum::response::Response, AppError> {
    download_video_impl(&state, &video_id).await
}

pub async fn download_shared_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Query(query): Query<SharedVideoQuery>,
) -> Result<axum::response::Response, AppError> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    let claims: crate::api::auth::ShareClaims = decode::<crate::api::auth::ShareClaims>(
        &query.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid share token: {}", e)))?;

    if claims.video_id != video_id {
        return Err(AppError::Unauthorized("Invalid share token for this video".to_string()));
    }

    download_video_impl(&state, &video_id).await
}


pub async fn regenerate_preview(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(video_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !_user.0.is_admin {
        return Err(AppError::Forbidden);
    }
    let db = state.db.clone();
    let vid_clone = video_id.clone();

    let seafile_path = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        
        let video = videos::table
            .filter(videos::id.eq(&vid_clone))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;
            
        diesel::update(videos::table.filter(videos::id.eq(&vid_clone)))
            .set(videos::preview_count.eq(0))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
            
        Ok::<String, AppError>(video.seafile_path)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let seafile = state.seafile.clone();
    let previews_dir = state.previews_dir.clone();
    let db = state.db.clone();
    let server_port = state.server_port;

    let ws_hub = state.ws_hub.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::services::previews::generate_previews(
            &video_id,
            &seafile,
            &seafile_path,
            std::path::Path::new(&previews_dir),
            &db,
            server_port,
        )
        .await
        {
            tracing::error!("generate_previews failed for {video_id}: {e:?}");
        } else {
            let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoPreview {
                video_id: video_id.clone(),
                preview_url: format!("/api/videos/{}/previews/0", video_id),
            });
        }
    });

    Ok(Json(serde_json::json!({ "status": "regenerating" })))
}

// ── Admin Database Sync ───────────────────────────────────────────────────────

pub async fn admin_sync_check(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<crate::db::models::Video>>, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let stale = crate::services::sync::check_stale_videos(&state.seafile, &state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(stale))
}

#[derive(Deserialize)]
pub struct AdminSyncCleanRequest {
    pub ids: Vec<String>,
}

pub async fn admin_sync_clean(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<AdminSyncCleanRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let count = body.ids.len();
    
    // Perform cascade deletion
    crate::services::sync::delete_videos_cascade(&state.db, &state.previews_dir, &body.ids)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Send WebSocket notification for each removed video
    for id in &body.ids {
        let _ = state.ws_hub.send(WsEvent::VideoRemoved { id: id.clone() });
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "deleted_count": count
    })))
}

async fn check_admin_token(
    headers: &HeaderMap,
    query_token: Option<&str>,
    jwt_secret: &str,
    db: &crate::db::DbPool,
) -> bool {
    let token_opt = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| query_token.map(|s| s.to_string()));

    if let Some(token) = token_opt {
        if let Ok(claims) = crate::api::auth::verify_token(&token, jwt_secret) {
            let user_id = claims.sub;
            let db_clone = db.clone();
            let res = tokio::task::spawn_blocking(move || {
                use crate::db::schema::users::dsl::{id, users};
                let mut conn = db_clone.get().map_err(|e| e.to_string())?;
                users.filter(id.eq(&user_id))
                    .first::<User>(&mut conn)
                    .optional()
                    .map_err(|e| e.to_string())
            }).await;
            if let Ok(Ok(Some(user))) = res {
                return user.is_admin;
            }
        }
    }
    false
}

#[derive(Deserialize)]
pub struct AdminImportQuery {
    pub key: Option<String>,
    pub token: Option<String>,
}

pub async fn admin_import_videos(
    State(state): State<AppState>,
    Query(query): Query<AdminImportQuery>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut authorized = false;

    // 1. Check IMPORT_TRIGGER_KEY from .env
    if let Ok(secret_key) = std::env::var("IMPORT_TRIGGER_KEY") {
        if let Some(ref req_key) = query.key {
            if req_key == &secret_key {
                authorized = true;
            }
        }
    }

    // 2. If key doesn't match, check regular Admin Bearer authorization
    if !authorized {
        let query_token = query.token.as_deref();
        if check_admin_token(&headers, query_token, &state.jwt_secret, &state.db).await {
            authorized = true;
        }
    }

    if !authorized {
        return Err(AppError::Forbidden);
    }

    // 3. Trigger video import
    crate::services::sync::import_new_videos(&state.seafile, &state.db, &state.ws_hub)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "ok"
    })))
}

// ── AI Label ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct WhisperExchange {
    start_ms: i32,
    end_ms: i32,
}

#[derive(Deserialize)]
struct WhisperResponse {
    #[allow(dead_code)]
    video_id: String,
    exchanges: Vec<WhisperExchange>,
}

pub async fn ai_label_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(video_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    // 1. Load video record
    let db = state.db.clone();
    let video_id_clone = video_id.clone();
    let video = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        videos::table
            .filter(videos::id.eq(&video_id_clone))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Check if video is human-labeled (only if NOT already AI labeled)
    if !video.is_ai_labeled {
        let db_check = state.db.clone();
        let video_id_check = video_id.clone();
        let is_human_labeled = tokio::task::spawn_blocking(move || {
            use crate::db::schema::bouts;
            let mut conn = db_check.get().map_err(|e| AppError::Internal(e.to_string()))?;
            let human_bouts_count: i64 = bouts::table
                .filter(bouts::video_id.eq(&video_id_check))
                .filter(bouts::is_ai.eq(false))
                .count()
                .get_result(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok::<bool, AppError>(human_bouts_count > 0)
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

        if is_human_labeled {
            return Err(AppError::BadRequest("Нельзя размечать с помощью ИИ видео, размеченное человеком".to_string()));
        }
    }

    // Set is_analyzing = true in DB
    let db_clone = state.db.clone();
    let video_id_db_init = video_id.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&video_id_db_init)))
            .set(videos::is_analyzing.eq(true))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Send initial WS message to show the spinner in all active clients
    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
        video_id: video_id.clone(),
        is_ai_labeled: false,
        is_analyzing: true,
    });

    // Spawn background worker to download and analyze the video
    let video_id_worker = video_id.clone();
    let ws_hub = state.ws_hub.clone();
    let seafile = state.seafile.clone();
    let db_for_worker = state.db.clone();
    
    tokio::spawn(async move {
        let run_analysis = || async {
            // Get download URL from Seafile
            let download_url = seafile
                .get_download_url(&video.seafile_path)
                .await
                .map_err(|e| format!("Seafile error: {}", e))?;

            // Contact whisper-service
            let whisper_url = std::env::var("WHISPER_URL")
                .unwrap_or_else(|_| "http://whisper-service:8000".to_string());

            let client = reqwest::Client::new();
            let resp = client
                .post(format!("{}/analyze", whisper_url))
                .json(&serde_json::json!({
                    "audio_url": download_url,
                    "video_id": video_id_worker
                }))
                .timeout(std::time::Duration::from_secs(3600))
                .send()
                .await
                .map_err(|e| format!("Whisper service unreachable: {}", e))?;

            let raw_json = resp
                .text()
                .await
                .map_err(|e| format!("Failed to read whisper response text: {}", e))?;

            // Save raw transcript response for admin viewer
            let _ = tokio::fs::create_dir_all("data/transcripts").await;
            let transcript_path = format!("data/transcripts/{}.json", video_id_worker);
            let _ = tokio::fs::write(&transcript_path, &raw_json).await;

            let whisper_resp: WhisperResponse = serde_json::from_str(&raw_json)
                .map_err(|e| format!("Failed to parse whisper response: {}", e))?;

            // Replace bouts in DB
            let exchanges = whisper_resp.exchanges;
            let video_id_db = video_id_worker.clone();
            let db_clone = db_for_worker.clone();
            
            tokio::task::spawn_blocking(move || {
                use crate::db::schema::{bouts, videos};
                use crate::db::models::NewBout;

                let mut conn = db_clone.get().map_err(|e| e.to_string())?;

                // Check if user cancelled analysis while it was running
                let is_still_analyzing: bool = videos::table
                    .filter(videos::id.eq(&video_id_db))
                    .select(videos::is_analyzing)
                    .first::<bool>(&mut conn)
                    .unwrap_or(false);

                if !is_still_analyzing {
                    return Err("AI analysis was cancelled by user".to_string());
                }

                // Delete existing bouts
                diesel::delete(bouts::table.filter(bouts::video_id.eq(&video_id_db)))
                    .execute(&mut conn)
                    .map_err(|e| e.to_string())?;

                // Insert new bouts
                for (i, ex) in exchanges.iter().enumerate() {
                    diesel::insert_into(bouts::table)
                        .values(&NewBout {
                            video_id: video_id_db.clone(),
                            order_index: (i + 1) as i32,
                            time_start_ms: ex.start_ms,
                            time_end_ms: ex.end_ms,
                            score_a: 0,
                            score_b: 0,
                            technique_a_id: None,
                            technique_b_id: None,
                            hit_zone_a: None,
                            hit_zone_b: None,
                            result_a: None,
                            result_b: None,
                            is_ai: true,
                        })
                        .execute(&mut conn)
                        .map_err(|e| e.to_string())?;
                }

                // Query the inserted bouts to get their generated IDs
                let inserted_bouts = bouts::table
                    .filter(bouts::video_id.eq(&video_id_db))
                    .load::<crate::db::models::Bout>(&mut conn)
                    .map_err(|e| e.to_string())?;

                // Insert history records for the AI-created bouts
                use crate::db::models::NewBoutHistory;
                use crate::db::schema::bout_history;
                use crate::api::bouts::format_ms;

                for bout in &inserted_bouts {
                    let details = format!(
                        "Время: {} — {}",
                        format_ms(bout.time_start_ms),
                        format_ms(bout.time_end_ms)
                    );

                    diesel::insert_into(bout_history::table)
                        .values(&NewBoutHistory {
                            bout_id: bout.id,
                            user_id: "ai".to_string(),
                            action: "create".to_string(),
                            details: Some(details),
                        })
                        .execute(&mut conn)
                        .map_err(|e| e.to_string())?;
                }

                // Mark video as AI-labeled and set is_analyzing to false
                diesel::update(videos::table.filter(videos::id.eq(&video_id_db)))
                    .set((
                        videos::is_ai_labeled.eq(true),
                        videos::is_analyzing.eq(false),
                    ))
                    .execute(&mut conn)
                    .map_err(|e| e.to_string())?;

                Ok::<(), String>(())
            })
            .await
            .map_err(|e| format!("Task join error: {}", e))?
        };

        match run_analysis().await {
            Ok(_) => {
                println!("AI labeling completed successfully for video {}", video_id_worker);
                let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
                    video_id: video_id_worker.clone(),
                    is_ai_labeled: true,
                    is_analyzing: false,
                });
                let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoScore {
                    video_id: video_id_worker,
                    total_score_a: 0,
                    total_score_b: 0,
                });
            }
            Err(err) => {
                eprintln!("AI labeling failed for video {}: {}", video_id_worker, err);
                
                // Reset is_analyzing in DB on failure
                let db_clone = db_for_worker.clone();
                let video_id_db = video_id_worker.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    use crate::db::schema::videos;
                    if let Ok(mut conn) = db_clone.get() {
                        let _ = diesel::update(videos::table.filter(videos::id.eq(&video_id_db)))
                            .set(videos::is_analyzing.eq(false))
                            .execute(&mut conn);
                    }
                }).await;

                let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
                    video_id: video_id_worker,
                    is_ai_labeled: false,
                    is_analyzing: false,
                });
            }
        }
    });

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "processing",
            "video_id": video_id
        })),
    ))
}

pub async fn cancel_ai_label_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(video_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let db_clone = state.db.clone();
    let video_id_db = video_id.clone();

    let is_ai_labeled = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&video_id_db)))
            .set(videos::is_analyzing.eq(false))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let v = videos::table
            .filter(videos::id.eq(&video_id_db))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        Ok::<bool, AppError>(v.is_ai_labeled)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
        video_id: video_id.clone(),
        is_ai_labeled,
        is_analyzing: false,
    });

    Ok(Json(serde_json::json!({ "status": "cancelled" })))
}

#[derive(Deserialize)]
pub struct TranscriptQuery {
    pub token: Option<String>,
}

pub async fn get_video_transcript(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Query(query): Query<TranscriptQuery>,
) -> Result<axum::response::Response, AppError> {
    let token = query.token.unwrap_or_default();
    let is_admin = match crate::api::auth::verify_token(&token, &state.jwt_secret) {
        Ok(claims) => {
            let db = state.db.clone();
            let uid = claims.sub;
            tokio::task::spawn_blocking(move || {
                use crate::db::schema::users;
                if let Ok(mut conn) = db.get() {
                    users::table
                        .filter(users::id.eq(&uid))
                        .first::<crate::db::models::User>(&mut conn)
                        .map(|u| u.is_admin)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .await
            .unwrap_or(false)
        }
        Err(_) => false,
    };

    if !is_admin {
        return Err(AppError::Forbidden);
    }

    let json_path = format!("data/transcripts/{}.json", video_id);
    let html_content = match tokio::fs::read_to_string(&json_path).await {
        Ok(raw_json) => render_transcript_html(&video_id, &token, &raw_json).await,
        Err(_) => format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>Расшифровка ИИ: {}</title>
    <style>
        body {{ background: #0f172a; color: #f8fafc; font-family: system-ui, sans-serif; padding: 40px; text-align: center; }}
        .card {{ background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 40px; border-radius: 12px; max-width: 600px; margin: 0 auto; }}
        h1 {{ color: #f59e0b; font-size: 1.5rem; }}
        p {{ color: #94a3b8; font-size: 0.95rem; line-height: 1.6; }}
    </style>
</head>
<body>
    <div class="card">
        <h1>Расшифровка ИИ отсутствует</h1>
        <p>Для видео <code>{}</code> файл расшифровки распознавания пока не создан или был выполнен по старой версии сервиса.</p>
    </div>
</body>
</html>"#,
            video_id, video_id
        ),
    };

    let response = axum::response::Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(html_content))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}

async fn render_transcript_html(video_id: &str, token: &str, raw_json: &str) -> String {
    let template_paths = [
        "scratch/exchange_viewer_target.html",
        "data/exchange_viewer_target.html",
        "backend/exchange_viewer_target.html",
        "exchange_viewer_target.html"
    ];
    let mut template = String::new();
    for p in template_paths {
        if let Ok(t) = tokio::fs::read_to_string(p).await {
            template = t;
            break;
        }
    }

    if template.is_empty() {
        return format!("<h1>Шаблон exchange_viewer_target.html не найден</h1>");
    }

    let parsed: serde_json::Value = serde_json::from_str(raw_json).unwrap_or(serde_json::Value::Null);

    let exchanges_json = if let Some(ex_arr) = parsed.get("exchanges").and_then(|v| v.as_array()) {
        let mut formatted = Vec::new();
        for (idx, ex) in ex_arr.iter().enumerate() {
            let start_sec = ex.get("start_ms").and_then(|v| v.as_f64()).map(|m| m / 1000.0)
                .or_else(|| ex.get("start_time_sec").and_then(|v| v.as_f64())).unwrap_or(0.0);
            let end_sec = ex.get("end_ms").and_then(|v| v.as_f64()).map(|m| m / 1000.0)
                .or_else(|| ex.get("end_time_sec").and_then(|v| v.as_f64())).unwrap_or(0.0);
            let text = ex.get("text")
                .or_else(|| ex.get("stop_word_detected"))
                .or_else(|| ex.get("stop_word"))
                .and_then(|v| v.as_str())
                .unwrap_or("—")
                .to_string();
            let conf = ex.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.8);
            let peak_sec = ex.get("peak_time_sec").and_then(|v| v.as_f64()).unwrap_or(start_sec + 2.0);
            let ratio = ex.get("peak_ratio").and_then(|v| v.as_f64()).unwrap_or(1.0);
            let is_ai = ex.get("is_ai").and_then(|v| v.as_bool()).unwrap_or(true);

            formatted.push(serde_json::json!({
                "exchange_id": idx + 1,
                "start_time_sec": start_sec,
                "end_time_sec": end_sec,
                "stop_word_detected": text,
                "confidence": conf,
                "peak_time_sec": peak_sec,
                "peak_ratio": ratio,
                "is_ai": is_ai
            }));
        }
        serde_json::to_string(&formatted).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    };

    let words_json = if let Some(w_arr) = parsed.get("words").or_else(|| parsed.get("allWords")).and_then(|v| v.as_array()) {
        serde_json::to_string(w_arr).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    };

    let stream_url = format!("/api/videos/{}/stream?token={}", video_id, token);

    let html = template
        .replace("<title>Errant Fox — Проверка разметки сходов</title>", &format!("<title>Errant Fox — Проверка разметки сходов: {}</title>", video_id))
        .replace("let EMBEDDED_EXCHANGES = [];", &format!("let EMBEDDED_EXCHANGES = {};", exchanges_json))
        .replace("let allWords = [];", &format!("let allWords = {};", words_json))
        .replace("let streamUrl = \"\";", &format!("let streamUrl = \"{}\";", stream_url));

    html
}


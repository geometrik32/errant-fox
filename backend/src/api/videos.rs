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
    pub is_queued: bool,
    pub is_optimized: bool,
    pub is_optimizing: bool,
    pub is_eligible_for_optimization: bool,
    pub has_transcript: bool,
    pub has_human_bouts: bool,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawing: Option<String>,
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
    pub is_queued: bool,
    pub is_optimized: bool,
    pub is_optimizing: bool,
    pub is_eligible_for_optimization: bool,
    pub has_transcript: bool,
    pub has_human_bouts: bool,
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
    transcripts_dir: &str,
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
                drawing: c.drawing.clone(),
            }
        })
        .collect();

    let has_transcript = std::path::Path::new(&format!("{}/{}.json", transcripts_dir, video.id)).exists();
    let has_human_bouts = bouts.iter().any(|b| !b.is_ai);
    let is_eligible_for_optimization = !bouts.is_empty() && bouts.iter().all(|b| !b.is_ai) && !video.is_optimized && !video.is_optimizing;

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
        is_queued: video.is_queued,
        is_optimized: video.is_optimized,
        is_optimizing: video.is_optimizing,
        is_eligible_for_optimization,
        has_transcript,
        has_human_bouts,
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
    let transcripts_dir = state.transcripts_dir.clone();

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

                let bouts = bouts_by_video
                    .get(&v.id)
                    .map(|b| b.as_slice())
                    .unwrap_or(&[]);

                let has_human_bouts = bouts.iter().any(|b| !b.is_ai);
                let is_eligible_for_optimization = !bouts.is_empty() && bouts.iter().all(|b| !b.is_ai) && !v.is_optimized && !v.is_optimizing;

                let (total_score_a, total_score_b) = if is_tagged {
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
                    is_queued: v.is_queued,
                    is_optimized: v.is_optimized,
                    is_optimizing: v.is_optimizing,
                    is_eligible_for_optimization,
                    has_transcript: std::path::Path::new(&format!("{}/{}.json", transcripts_dir, v.id)).exists(),
                    has_human_bouts,
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
    let transcripts_dir = state.transcripts_dir.clone();
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
            &transcripts_dir,
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
    pub drawing: Option<String>,
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

        use crate::db::schema::comments;

        let effective_reply_to_id = match body.reply_to_id {
            Some(pid) => {
                if let Ok(parent) = comments::table.filter(comments::id.eq(pid)).first::<Comment>(&mut conn) {
                    Some(parent.reply_to_id.unwrap_or(pid))
                } else {
                    Some(pid)
                }
            }
            None => None,
        };

        // 2. Insert new comment
        let new_comment = NewComment {
            video_id: video_id.clone(),
            author_id: user.id.clone(),
            timestamp_ms: body.timestamp_ms,
            text: body.text,
            reply_to_id: effective_reply_to_id,
            guest_nickname: Some(nickname.clone()),
            drawing: body.drawing.clone(),
        };

        let c: Comment = conn.transaction::<Comment, diesel::result::Error, _>(|tx_conn| {
            diesel::insert_into(comments::table)
                .values(&new_comment)
                .execute(tx_conn)?;
            let comment_id: i32 = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
                .get_result(tx_conn)?;
            comments::table.filter(comments::id.eq(comment_id)).first::<Comment>(tx_conn)
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
            drawing: c.drawing.clone(),
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
            drawing: c.drawing.clone(),
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
    let transcripts_dir = state.transcripts_dir.clone();

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
            &transcripts_dir,
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

// ── Video stream proxy & on-demand transcode ──────────────────────────────────

#[derive(Deserialize, Default)]
pub struct StreamQuery {
    pub codec: Option<String>,
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct StreamStatusQuery {
    pub codec: Option<String>,
    pub auto_start: Option<bool>,
}

#[derive(Serialize)]
pub struct StreamStatusDto {
    pub ready: bool,
    pub in_progress: bool,
}

pub async fn stream_status(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Query(query): Query<StreamStatusQuery>,
) -> Result<Json<StreamStatusDto>, AppError> {
    if query.codec.as_deref() == Some("h264") {
        let (ready, in_progress) = state.transcode.check_status(&video_id).await;
        if !ready && !in_progress && query.auto_start.unwrap_or(true) {
            let _ = state.transcode.start_or_subscribe(&video_id).await;
            return Ok(Json(StreamStatusDto {
                ready: false,
                in_progress: true,
            }));
        }
        return Ok(Json(StreamStatusDto {
            ready,
            in_progress,
        }));
    }

    Ok(Json(StreamStatusDto {
        ready: true,
        in_progress: false,
    }))
}

async fn serve_local_file_range(
    file_path: &std::path::Path,
    range_header: Option<&str>,
) -> Result<axum::response::Response, AppError> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let total_size = metadata.len();

    let (start, end) = if let Some(range_str) = range_header {
        if let Some(bytes_range) = range_str.strip_prefix("bytes=") {
            let parts: Vec<&str> = bytes_range.split('-').collect();
            let start = parts.first().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
            let end = parts
                .get(1)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(total_size.saturating_sub(1));
            let end = end.min(total_size.saturating_sub(1));
            (start, end)
        } else {
            (0, total_size.saturating_sub(1))
        }
    } else {
        (0, total_size.saturating_sub(1))
    };

    let length = if total_size == 0 || start > end {
        0
    } else {
        end - start + 1
    };

    file.seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let chunk_size = 64 * 1024;
    let stream = futures_util::stream::unfold(
        (file, length),
        move |(mut f, mut remaining)| async move {
            if remaining == 0 {
                return None;
            }
            let to_read = (chunk_size as u64).min(remaining) as usize;
            let mut buf = vec![0u8; to_read];
            match f.read_exact(&mut buf).await {
                Ok(_) => {
                    remaining -= to_read as u64;
                    Some((Ok::<_, std::io::Error>(axum::body::Bytes::from(buf)), (f, remaining)))
                }
                Err(e) => Some((Err(e), (f, 0))),
            }
        },
    );

    let body = axum::body::Body::from_stream(stream);

    let mut builder = axum::response::Response::builder()
        .header("content-type", "video/mp4")
        .header("accept-ranges", "bytes");

    if range_header.is_some() {
        builder = builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(
                "content-range",
                format!("bytes {}-{}/{}", start, end, total_size),
            )
            .header("content-length", length.to_string());
    } else {
        builder = builder
            .status(StatusCode::OK)
            .header("content-length", total_size.to_string());
    }

    builder
        .body(body)
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn stream_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Query(query): Query<StreamQuery>,
    req: axum::extract::Request,
) -> Result<axum::response::Response, AppError> {
    // If client requests H.264 compatible stream
    if query.codec.as_deref() == Some("h264") {
        let target_file = state.transcode.target_path(&video_id);
        if target_file.exists() {
            state.transcode.touch(&video_id).await;
            let range = req
                .headers()
                .get(header::RANGE)
                .and_then(|v| v.to_str().ok());
            return serve_local_file_range(&target_file, range).await;
        } else {
            // Not ready yet — trigger transcode if not already running
            let _ = state.transcode.start_or_subscribe(&video_id).await;
            return Err(AppError::NotFound);
        }
    }

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

    let range = req
        .headers()
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

#[derive(Serialize)]
pub struct AdminSyncCheckResult {
    pub imported_count: usize,
    pub stale: Vec<crate::db::models::Video>,
}

pub async fn admin_sync_check(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<AdminSyncCheckResult>, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let imported_ids = crate::services::sync::import_new_videos(&state.seafile, &state.db, &state.ws_hub)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let stale = crate::services::sync::check_stale_videos(&state.seafile, &state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(AdminSyncCheckResult {
        imported_count: imported_ids.len(),
        stale,
    }))
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
    pub ai_label: Option<bool>,
}

pub async fn admin_import_videos(
    State(state): State<AppState>,
    Query(query): Query<AdminImportQuery>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut authorized = false;

    // 1. Check secret key against state.jwt_secret
    if let Some(ref req_key) = query.key {
        if req_key == &state.jwt_secret {
            authorized = true;
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
    let imported_ids = crate::services::sync::import_new_videos(&state.seafile, &state.db, &state.ws_hub)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut ai_queued_count = 0;
    if query.ai_label == Some(true) && !imported_ids.is_empty() {
        let db_batch = state.db.clone();
        let target_ids = imported_ids.clone();
        let _ = tokio::task::spawn_blocking(move || {
            use crate::db::schema::videos;
            if let Ok(mut conn) = db_batch.get() {
                let _ = diesel::update(videos::table.filter(videos::id.eq_any(&target_ids)))
                    .set(videos::is_queued.eq(true))
                    .execute(&mut conn);
            }
        }).await;

        for vid in &imported_ids {
            let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
                video_id: vid.clone(),
                is_ai_labeled: false,
                is_analyzing: false,
                is_queued: true,
                has_transcript: None,
            });
            let _ = state.ai_queue_tx.send(vid.clone());
        }
        ai_queued_count = imported_ids.len();
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "imported_count": imported_ids.len(),
        "imported_ids": imported_ids,
        "ai_queued_count": ai_queued_count
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

pub async fn execute_ai_label_for_video(state: AppState, video_id: String) -> Result<(), AppError> {
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

    if video.is_analyzing {
        return Ok(());
    }

    // Check if video is human-labeled
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
            let db_clean = state.db.clone();
            let video_id_clean = video_id.clone();
            let _ = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                if let Ok(mut conn) = db_clean.get() {
                    let _ = diesel::update(videos::table.filter(videos::id.eq(&video_id_clean)))
                        .set((
                            videos::is_analyzing.eq(false),
                            videos::is_queued.eq(false),
                        ))
                        .execute(&mut conn);
                }
            }).await;
            let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
                video_id: video_id.clone(),
                is_ai_labeled: false,
                is_analyzing: false,
                is_queued: false,
                has_transcript: None,
            });
            return Err(AppError::BadRequest("Нельзя размечать с помощью ИИ видео, размеченное человеком".to_string()));
        }
    }

    // Set is_analyzing = true, is_queued = false in DB
    let db_clone = state.db.clone();
    let video_id_db_init = video_id.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&video_id_db_init)))
            .set((
                videos::is_analyzing.eq(true),
                videos::is_queued.eq(false),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Send initial WS message
    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
        video_id: video_id.clone(),
        is_ai_labeled: false,
        is_analyzing: true,
        is_queued: false,
        has_transcript: None,
    });

    let video_id_worker = video_id.clone();
    let ws_hub = state.ws_hub.clone();
    let seafile = state.seafile.clone();
    let db_for_worker = state.db.clone();

    let run_analysis = || async {
        let whisper_url = std::env::var("WHISPER_URL")
            .unwrap_or_else(|_| "http://whisper-service:8000".to_string());

        let download_url = seafile
            .get_download_url(&video.seafile_path)
            .await
            .map_err(|e| format!("Seafile error: {}", e))?;

        let client = reqwest::Client::new();
        let mut retry_count = 0;
        let resp = loop {
            match client
                .post(format!("{}/analyze", whisper_url))
                .json(&serde_json::json!({
                    "audio_url": download_url,
                    "video_id": video_id_worker
                }))
                .timeout(std::time::Duration::from_secs(900))
                .send()
                .await
            {
                Ok(r) => break r,
                Err(e) => {
                    if e.is_timeout() || retry_count >= 12 {
                        return Err(format!("Whisper service unreachable: {}", e));
                    }
                    retry_count += 1;
                    println!("Whisper service not ready, waiting 5s... ({}/12)", retry_count);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let detail = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("detail").and_then(|d| d.as_str()).map(|s| s.to_string()))
                .unwrap_or(text);
            return Err(format!("Whisper service error (HTTP {}): {}", status, detail));
        }

        let raw_json = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read whisper response text: {}", e))?;

        let transcripts_dir = state.transcripts_dir.clone();
        let _ = tokio::fs::create_dir_all(&transcripts_dir).await;
        let transcript_path = format!("{}/{}.json", transcripts_dir, video_id_worker);
        let _ = tokio::fs::write(&transcript_path, &raw_json).await;

        let whisper_resp: WhisperResponse = serde_json::from_str(&raw_json)
            .map_err(|e| format!("Failed to parse whisper response: {}", e))?;

        let exchanges = whisper_resp.exchanges;
        let video_id_db = video_id_worker.clone();
        let db_clone = db_for_worker.clone();

        tokio::task::spawn_blocking(move || {
            use crate::db::schema::{bouts, videos};
            use crate::db::models::NewBout;

            let mut conn = db_clone.get().map_err(|e| e.to_string())?;

            let is_still_analyzing: bool = videos::table
                .filter(videos::id.eq(&video_id_db))
                .select(videos::is_analyzing)
                .first::<bool>(&mut conn)
                .unwrap_or(false);

            if !is_still_analyzing {
                return Err("AI analysis was cancelled by user".to_string());
            }

            diesel::delete(bouts::table.filter(bouts::video_id.eq(&video_id_db)))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;

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

            let inserted_bouts = bouts::table
                .filter(bouts::video_id.eq(&video_id_db))
                .load::<crate::db::models::Bout>(&mut conn)
                .map_err(|e| e.to_string())?;

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

            diesel::update(videos::table.filter(videos::id.eq(&video_id_db)))
                .set((
                    videos::is_ai_labeled.eq(true),
                    videos::is_analyzing.eq(false),
                    videos::is_queued.eq(false),
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
                is_queued: false,
                has_transcript: Some(true),
            });
            let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoScore {
                video_id: video_id_worker,
                total_score_a: 0,
                total_score_b: 0,
            });
        }
        Err(err) => {
            eprintln!("AI labeling failed for video {}: {}", video_id_worker, err);
            let db_clone = db_for_worker.clone();
            let video_id_db = video_id_worker.clone();
            let _ = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                if let Ok(mut conn) = db_clone.get() {
                    let _ = diesel::update(videos::table.filter(videos::id.eq(&video_id_db)))
                        .set((
                            videos::is_analyzing.eq(false),
                            videos::is_queued.eq(false),
                        ))
                        .execute(&mut conn);
                }
            }).await;

            let _ = ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
                video_id: video_id_worker,
                is_ai_labeled: false,
                is_analyzing: false,
                is_queued: false,
                has_transcript: None,
            });
        }
    }

    Ok(())
}

pub async fn ai_label_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(video_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let db_clone = state.db.clone();
    let video_id_db = video_id.clone();

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&video_id_db)))
            .set(videos::is_queued.eq(true))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
        video_id: video_id.clone(),
        is_ai_labeled: false,
        is_analyzing: false,
        is_queued: true,
        has_transcript: None,
    });

    let _ = state.ai_queue_tx.send(video_id);

    Ok(Json(serde_json::json!({ "status": "queued" })))
}

#[derive(Deserialize)]
pub struct BatchAiLabelRequest {
    pub video_ids: Option<Vec<String>>,
}

pub async fn batch_ai_label_video(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<BatchAiLabelRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let raw_ids = if let Some(ids) = body.video_ids {
        ids
    } else {
        tokio::task::spawn_blocking(move || {
            use crate::db::schema::{videos, bouts};
            let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

            let all_videos = videos::table
                .select((videos::id, videos::is_ai_labeled, videos::is_analyzing))
                .load::<(String, bool, bool)>(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let mut unanalyzed = Vec::new();
            for (v_id, is_ai_labeled, is_analyzing) in all_videos {
                if !is_ai_labeled && !is_analyzing {
                    let human_bouts: i64 = bouts::table
                        .filter(bouts::video_id.eq(&v_id))
                        .filter(bouts::is_ai.eq(false))
                        .count()
                        .get_result(&mut conn)
                        .unwrap_or(0);
                    if human_bouts == 0 {
                        unanalyzed.push(v_id);
                    }
                }
            }
            Ok::<Vec<String>, AppError>(unanalyzed)
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??
    };

    // Filter out any video that has human-labeled bouts or is currently analyzing
    let db_filter = state.db.clone();
    let video_ids: Vec<String> = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{videos, bouts};
        let mut conn = db_filter.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let target_videos = videos::table
            .filter(videos::id.eq_any(&raw_ids))
            .select((videos::id, videos::is_ai_labeled, videos::is_analyzing))
            .load::<(String, bool, bool)>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut valid = Vec::new();
        for (v_id, is_ai_labeled, is_analyzing) in target_videos {
            if is_analyzing {
                continue;
            }
            if is_ai_labeled {
                valid.push(v_id);
            } else {
                let human_bouts: i64 = bouts::table
                    .filter(bouts::video_id.eq(&v_id))
                    .filter(bouts::is_ai.eq(false))
                    .count()
                    .get_result(&mut conn)
                    .unwrap_or(0);
                if human_bouts == 0 {
                    valid.push(v_id);
                }
            }
        }
        Ok::<Vec<String>, AppError>(valid)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let count = video_ids.len();

    // Mark all target videos as is_queued = true in DB
    let db_batch = state.db.clone();
    let target_ids = video_ids.clone();
    let _ = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        if let Ok(mut conn) = db_batch.get() {
            let _ = diesel::update(videos::table.filter(videos::id.eq_any(&target_ids)))
                .set(videos::is_queued.eq(true))
                .execute(&mut conn);
        }
    }).await;

    // Send WS notification for every queued video & send to queue worker
    for vid in &video_ids {
        let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
            video_id: vid.clone(),
            is_ai_labeled: false,
            is_analyzing: false,
            is_queued: true,
            has_transcript: None,
        });
        let _ = state.ai_queue_tx.send(vid.clone());
    }

    Ok(Json(serde_json::json!({
        "status": "batch_started",
        "count": count
    })))
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
            .set((
                videos::is_analyzing.eq(false),
                videos::is_queued.eq(false),
            ))
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
        is_queued: false,
        has_transcript: None,
    });

    // Notify whisper-service to skip this video if it's queued
    let whisper_url = std::env::var("WHISPER_URL")
        .unwrap_or_else(|_| "http://whisper-service:8000".to_string());
    let video_id_cancel = video_id.clone();
    tokio::spawn(async move {
        let _ = reqwest::Client::new()
            .post(format!("{}/cancel/{}", whisper_url, video_id_cancel))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
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

    let json_path = format!("{}/{}.json", state.transcripts_dir, video_id);
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

#[derive(Deserialize)]
pub struct ShareOgQuery {
    pub token: Option<String>,
    pub bout_id: Option<i32>,
    pub t: Option<i64>,
}

pub async fn og_share_video(
    Path(id): Path<String>,
    Query(query): Query<ShareOgQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let video_id = id.clone();
    let db_pool = state.db.clone();
    let bout_id_opt = query.bout_id;

    let res = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{videos, users, bouts};
        let mut conn = db_pool.get().ok()?;

        let video = videos::table
            .filter(videos::id.eq(&video_id))
            .first::<Video>(&mut conn)
            .ok()?;

        let fighter_a = video.fighter_a_id.as_ref().and_then(|fa_id| {
            users::table.filter(users::id.eq(fa_id)).first::<User>(&mut conn).ok()
        });
        let fighter_b = video.fighter_b_id.as_ref().and_then(|fb_id| {
            users::table.filter(users::id.eq(fb_id)).first::<User>(&mut conn).ok()
        });

        let mut bout_info = None;
        if let Some(bid) = bout_id_opt {
            if let Ok(b) = bouts::table.filter(bouts::id.eq(bid)).first::<Bout>(&mut conn) {
                bout_info = Some(b);
            }
        }

        Some((video, fighter_a, fighter_b, bout_info))
    }).await.ok().flatten();

    let (title, description) = match res {
        Some((video, fighter_a, fighter_b, bout_info)) => {
            let name_a = fighter_a.map(|u| u.display_name).unwrap_or_else(|| "Игрок A".to_string());
            let name_b = fighter_b.map(|u| u.display_name).unwrap_or_else(|| "Игрок B".to_string());
            let base_title = format!("{} vs {}", name_a, name_b);

            let main_title = if let Some(b) = bout_info {
                format!("Сход №{} • Errant Fox — {}", b.order_index, base_title)
            } else {
                format!("Errant Fox — {}", base_title)
            };

            let desc = format!("Анализ фехтовального боя. Дата: {}", video.date);
            (main_title, desc)
        }
        None => (
            "Errant Fox — Видео".to_string(),
            "Анализ фехтовальных боев и сходов в Errant Fox".to_string(),
        )
    };

    let token_param = query.token.as_deref().unwrap_or("");
    let bout_param = query.bout_id.map(|b| format!("&bout_id={}", b)).unwrap_or_default();
    let time_param = query.t.map(|t| format!("&t={}", t)).unwrap_or_default();

    let target_hash_url = format!("/#/share/video/{}?token={}{}{}", id, token_param, bout_param, time_param);

    let html = format!(r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <meta property="og:site_name" content="Errant Fox" />
    <meta property="og:type" content="video.other" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:image" content="/icon-512.png" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <meta http-equiv="refresh" content="0;url={target_hash_url}" />
</head>
<body style="background:#0a0a0c;color:#fff;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;">
    <div style="text-align:center;">
        <p>Перенаправление на Errant Fox...</p>
        <p><a href="{target_hash_url}" style="color:#f59e0b;">Нажмите здесь, если перенаправление не произошло автоматически</a></p>
    </div>
    <script>
        window.location.href = "{target_hash_url}";
    </script>
</body>
</html>"#);

    axum::response::Html(html)
}

pub async fn get_trim_ui(
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

    let template_paths = [
        "scratch/video_trimmer_target.html",
        "data/video_trimmer_target.html",
        "backend/video_trimmer_target.html",
        "video_trimmer_target.html"
    ];
    let mut template = String::new();
    for p in template_paths {
        if let Ok(t) = tokio::fs::read_to_string(p).await {
            template = t;
            break;
        }
    }

    if template.is_empty() {
        return Ok(axum::response::Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from("<h1>Шаблон video_trimmer_target.html не найден</h1>"))
            .unwrap());
    }

    let json_path = format!("{}/{}.json", state.transcripts_dir, video_id);
    let exchanges_json = match tokio::fs::read_to_string(&json_path).await {
        Ok(raw_json) => {
            let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap_or(serde_json::Value::Null);
            if let Some(ex_arr) = parsed.get("exchanges").and_then(|v| v.as_array()) {
                let mut formatted = Vec::new();
                for (idx, ex) in ex_arr.iter().enumerate() {
                    let start_sec = ex.get("start_ms").and_then(|v| v.as_f64()).map(|m| m / 1000.0)
                        .or_else(|| ex.get("start_time_sec").and_then(|v| v.as_f64())).unwrap_or(0.0);
                    let end_sec = ex.get("end_ms").and_then(|v| v.as_f64()).map(|m| m / 1000.0)
                        .or_else(|| ex.get("end_time_sec").and_then(|v| v.as_f64())).unwrap_or(0.0);
                    formatted.push(serde_json::json!({
                        "exchange_id": idx + 1,
                        "start_time_sec": start_sec,
                        "end_time_sec": end_sec,
                    }));
                }
                serde_json::to_string(&formatted).unwrap_or_else(|_| "[]".to_string())
            } else {
                "[]".to_string()
            }
        },
        Err(_) => {
            "[]".to_string()
        }
    };

    let stream_url = format!("/api/videos/{}/stream?token={}", video_id, token);
    
    let html = template
        .replace("let EMBEDDED_EXCHANGES = [];", &format!("let EMBEDDED_EXCHANGES = {};", exchanges_json))
        .replace("let streamUrl = \"\";", &format!("let streamUrl = \"{}\";", stream_url))
        .replace("let videoId = \"\";", &format!("let videoId = \"{}\";", video_id))
        .replace("let tokenStr = \"\";", &format!("let tokenStr = \"{}\";", token));

    let response = axum::response::Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(html))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct TrimVideoPayload {
    pub start_sec: f64,
    pub end_sec: f64,
}

#[derive(Clone, serde::Serialize)]
pub struct TrimStatusResponse {
    pub status: String,
    pub error: Option<String>,
}

static TRIM_JOBS: std::sync::LazyLock<std::sync::Arc<tokio::sync::RwLock<HashMap<String, TrimStatusResponse>>>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())));

pub async fn get_trim_status(
    State(_state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    Path(video_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    let jobs = TRIM_JOBS.read().await;
    let status = jobs.get(&video_id).cloned().unwrap_or(TrimStatusResponse {
        status: "idle".to_string(),
        error: None,
    });
    Ok(axum::Json(status))
}

pub async fn get_keyframes(
    State(state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    Path(video_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let temp_dir = get_temp_dir().await;
    let cache_file = format!("{}/keyframes_{}.json", temp_dir, video_id);

    if let Ok(data) = tokio::fs::read_to_string(&cache_file).await {
        if let Ok(kfs) = serde_json::from_str::<Vec<f64>>(&data) {
            return Ok(axum::Json(kfs));
        }
    }

    let db = state.db.clone();
    let vid = video_id.clone();
    let video = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;
        videos::table
            .filter(videos::id.eq(&vid))
            .first::<Video>(&mut conn)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|_| AppError::NotFound)?;

    let download_url = state
        .seafile
        .get_download_url(&video.seafile_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let output = tokio::process::Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-skip_frame")
        .arg("nokey")
        .arg("-show_entries")
        .arg("frame=pts_time")
        .arg("-of")
        .arg("csv=p=0")
        .arg("-user_agent")
        .arg("Mozilla/5.0")
        .arg(&download_url)
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to spawn ffprobe: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("ffprobe get_keyframes failed: {}", stderr);
        return Err(AppError::Internal(format!("ffprobe failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut keyframes: Vec<f64> = stdout
        .lines()
        .filter_map(|l| l.trim().parse::<f64>().ok())
        .collect();

    if keyframes.is_empty() || keyframes.first() != Some(&0.0) {
        keyframes.insert(0, 0.0);
    }
    keyframes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    keyframes.dedup();

    if let Ok(json_str) = serde_json::to_string(&keyframes) {
        let _ = tokio::fs::write(&cache_file, json_str).await;
    }

    Ok(axum::Json(keyframes))
}

async fn get_temp_dir() -> String {
    if let Ok(dir) = std::env::var("TEMP_DIR") {
        if tokio::fs::create_dir_all(&dir).await.is_ok() {
            return dir;
        }
    }
    let candidates = ["data/temp", "/data/temp", "/tmp/errant_fox_temp"];
    for candidate in candidates {
        if tokio::fs::create_dir_all(candidate).await.is_ok() {
            return candidate.to_string();
        }
    }
    std::env::temp_dir().to_string_lossy().to_string()
}

pub async fn trim_video(
    State(state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    Path(video_id): Path<String>,
    axum::Json(payload): axum::Json<TrimVideoPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    
    {
        let mut jobs = TRIM_JOBS.write().await;
        if let Some(job) = jobs.get(&video_id) {
            if job.status == "processing" {
                return Ok(axum::Json(serde_json::json!({ "status": "processing", "message": "Already processing" })));
            }
        }
        jobs.insert(video_id.clone(), TrimStatusResponse {
            status: "processing".to_string(),
            error: None,
        });
    }

    let vid = video_id.clone();
    tokio::spawn(async move {
        let res = run_trim_task(state, vid.clone(), payload).await;
        let mut jobs = TRIM_JOBS.write().await;
        match res {
            Ok(()) => {
                jobs.insert(vid, TrimStatusResponse {
                    status: "success".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                tracing::error!("Trim background task failed: {:?}", e);
                jobs.insert(vid, TrimStatusResponse {
                    status: "error".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }
    });

    Ok(axum::Json(serde_json::json!({ "status": "processing" })))
}

async fn run_trim_task(state: AppState, video_id: String, payload: TrimVideoPayload) -> Result<(), anyhow::Error> {
    let db = state.db.clone();
    
    // 1. Get video
    let video = tokio::task::spawn_blocking({
        let vid = video_id.clone();
        let db = db.clone();
        move || {
            use crate::db::schema::videos;
            use diesel::prelude::*;
            let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;
            videos::table
                .filter(videos::id.eq(&vid))
                .first::<Video>(&mut conn)
        }
    })
    .await??;

    let seafile_path = video.seafile_path.clone();
    
    let download_url = state.seafile.get_download_url(&seafile_path).await?;
    
    let temp_dir = get_temp_dir().await;
    let temp_input = format!("{}/input_{}.mp4", temp_dir, video_id);
    let temp_output = format!("{}/trimmed_{}.mp4", temp_dir, video_id);
    
    // Step 1: Download the original video
    tracing::info!("Trim: downloading video {} from Seafile to {}...", video_id, temp_input);
    let response = reqwest::get(&download_url).await?;
    {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&temp_input).await?;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }
    }
    tracing::info!("Trim: download complete, running ffmpeg...");
    
    // Step 2: Run ffmpeg lossless stream copy (-c copy) on keyframe bounds
    let duration = payload.end_sec - payload.start_sec;
    
    let output = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg(payload.start_sec.to_string())
        .arg("-i")
        .arg(&temp_input)
        .arg("-t")
        .arg(duration.to_string())
        .arg("-c")
        .arg("copy")
        .arg("-avoid_negative_ts")
        .arg("make_zero")
        .arg("-movflags")
        .arg("+faststart")
        .arg(&temp_output)
        .output()
        .await?;

    let _ = tokio::fs::remove_file(&temp_input).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("FFmpeg failed: {}", stderr);
        let _ = tokio::fs::remove_file(&temp_output).await;
        return Err(anyhow::anyhow!("FFmpeg trim failed: {}", stderr));
    }
    tracing::info!("Trim: ffmpeg stream copy done, uploading to Seafile...");

    // Step 3: Upload trimmed file back to Seafile
    let file_bytes = tokio::fs::read(&temp_output).await?;
    state.seafile.upload_file(&seafile_path, file_bytes).await?;
    let _ = tokio::fs::remove_file(&temp_output).await;

    // Invalidate stale caches for this trimmed video
    let _ = tokio::fs::remove_file(format!("{}/keyframes_{}.json", temp_dir, video_id)).await;
    let _ = tokio::fs::remove_file(state.transcode.target_path(&video_id)).await;

    tracing::info!("Trim: upload done, updating DB...");

    let start_ms = (payload.start_sec * 1000.0).round() as i32;
    let end_ms = (payload.end_sec * 1000.0).round() as i32;
    let new_duration_ms = ((payload.end_sec - payload.start_sec) * 1000.0).round() as i32;
    
    tokio::task::spawn_blocking({
        let vid = video_id.clone();
        let db = db.clone();
        move || -> Result<(), diesel::result::Error> {
            use crate::db::schema::{bouts, comments, videos};
            use diesel::prelude::*;
            let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;
            
            // Delete bouts that fall out of the kept range
            diesel::delete(
                bouts::table.filter(
                    bouts::video_id.eq(&vid)
                        .and(bouts::time_start_ms.lt(start_ms).or(bouts::time_end_ms.gt(end_ms)))
                )
            ).execute(&mut conn)?;

            // Shift remaining bouts
            diesel::update(bouts::table.filter(bouts::video_id.eq(&vid)))
                .set((
                    bouts::time_start_ms.eq(bouts::time_start_ms - start_ms),
                    bouts::time_end_ms.eq(bouts::time_end_ms - start_ms),
                ))
                .execute(&mut conn)?;

            // Delete comments that fall out of bounds
            diesel::delete(
                comments::table.filter(
                    comments::video_id.eq(&vid)
                        .and(comments::timestamp_ms.lt(start_ms).or(comments::timestamp_ms.gt(end_ms)))
                )
            ).execute(&mut conn)?;

            // Shift remaining comments
            diesel::update(comments::table.filter(comments::video_id.eq(&vid)))
                .set(comments::timestamp_ms.eq(comments::timestamp_ms - start_ms))
                .execute(&mut conn)?;

            // Update video duration
            diesel::update(videos::table.filter(videos::id.eq(&vid)))
                .set(videos::duration_ms.eq(Some(new_duration_ms)))
                .execute(&mut conn)?;

            Ok(())
        }
    })
    .await??;

    // 4. Update transcript JSON timestamps and words
    let json_path = format!("{}/{}.json", state.transcripts_dir, video_id);
    let mut has_transcript = false;
    if let Ok(raw_json) = tokio::fs::read_to_string(&json_path).await {
        if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&raw_json) {
            let start_sec = payload.start_sec;
            let end_sec = payload.end_sec;
            let start_ms = (start_sec * 1000.0).round() as i64;
            let end_ms = (end_sec * 1000.0).round() as i64;

            if let Some(exchanges) = parsed.get_mut("exchanges").and_then(|v| v.as_array_mut()) {
                let mut updated_exchanges = Vec::new();
                for ex in exchanges.iter_mut() {
                    let cur_start_ms = ex.get("start_ms").and_then(|v| v.as_i64())
                        .or_else(|| ex.get("start_time_sec").and_then(|v| v.as_f64()).map(|s| (s * 1000.0).round() as i64))
                        .unwrap_or(0);
                    let cur_end_ms = ex.get("end_ms").and_then(|v| v.as_i64())
                        .or_else(|| ex.get("end_time_sec").and_then(|v| v.as_f64()).map(|s| (s * 1000.0).round() as i64))
                        .unwrap_or(0);

                    // Keep only if exchange is within kept range
                    if cur_start_ms >= start_ms && cur_end_ms <= end_ms {
                        let new_start_ms = cur_start_ms - start_ms;
                        let new_end_ms = cur_end_ms - start_ms;
                        if let Some(obj) = ex.as_object_mut() {
                            obj.insert("start_ms".to_string(), serde_json::json!(new_start_ms));
                            obj.insert("end_ms".to_string(), serde_json::json!(new_end_ms));
                            obj.insert("start_time_sec".to_string(), serde_json::json!((new_start_ms as f64) / 1000.0));
                            obj.insert("end_time_sec".to_string(), serde_json::json!((new_end_ms as f64) / 1000.0));
                            if let Some(peak) = obj.get("peak_time_sec").and_then(|v| v.as_f64()) {
                                obj.insert("peak_time_sec".to_string(), serde_json::json!(peak - start_sec));
                            }
                        }
                        updated_exchanges.push(ex.clone());
                    }
                }
                parsed["exchanges"] = serde_json::json!(updated_exchanges);
            }

            // Also shift and filter words if present
            for words_key in &["words", "allWords"] {
                if let Some(words) = parsed.get_mut(*words_key).and_then(|v| v.as_array_mut()) {
                    let mut updated_words = Vec::new();
                    for w in words.iter_mut() {
                        let w_start_sec = w.get("start").and_then(|v| v.as_f64())
                            .or_else(|| w.get("start_time_sec").and_then(|v| v.as_f64()))
                            .or_else(|| w.get("start_ms").and_then(|v| v.as_f64()).map(|m| m / 1000.0));
                        let w_end_sec = w.get("end").and_then(|v| v.as_f64())
                            .or_else(|| w.get("end_time_sec").and_then(|v| v.as_f64()))
                            .or_else(|| w.get("end_ms").and_then(|v| v.as_f64()).map(|m| m / 1000.0));

                        if let (Some(w_start), Some(w_end)) = (w_start_sec, w_end_sec) {
                            if w_start >= start_sec && w_end <= end_sec {
                                if let Some(obj) = w.as_object_mut() {
                                    let new_w_start = (w_start - start_sec).max(0.0);
                                    let new_w_end = (w_end - start_sec).max(0.0);
                                    if obj.contains_key("start") {
                                        obj.insert("start".to_string(), serde_json::json!(new_w_start));
                                    }
                                    if obj.contains_key("end") {
                                        obj.insert("end".to_string(), serde_json::json!(new_w_end));
                                    }
                                    if obj.contains_key("start_time_sec") {
                                        obj.insert("start_time_sec".to_string(), serde_json::json!(new_w_start));
                                    }
                                    if obj.contains_key("end_time_sec") {
                                        obj.insert("end_time_sec".to_string(), serde_json::json!(new_w_end));
                                    }
                                    if obj.contains_key("start_ms") {
                                        obj.insert("start_ms".to_string(), serde_json::json!((new_w_start * 1000.0).round() as i64));
                                    }
                                    if obj.contains_key("end_ms") {
                                        obj.insert("end_ms".to_string(), serde_json::json!((new_w_end * 1000.0).round() as i64));
                                    }
                                }
                                updated_words.push(w.clone());
                            }
                        }
                    }
                    parsed[*words_key] = serde_json::json!(updated_words);
                }
            }

            if let Ok(updated_json) = serde_json::to_string_pretty(&parsed) {
                let _ = tokio::fs::write(&json_path, &updated_json).await;
                has_transcript = true;
            }
        }
    }

    // 5. Recalculate is_ai_labeled based on remaining AI bouts
    let is_ai = tokio::task::spawn_blocking({
        let vid = video_id.clone();
        let db = db.clone();
        move || -> Result<bool, diesel::result::Error> {
            use crate::db::schema::{bouts, videos};
            use diesel::prelude::*;
            let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;
            let ai_bouts_count: i64 = bouts::table
                .filter(bouts::video_id.eq(&vid))
                .filter(bouts::is_ai.eq(true))
                .count()
                .get_result(&mut conn)?;
            
            let is_ai = ai_bouts_count > 0;
            diesel::update(videos::table.filter(videos::id.eq(&vid)))
                .set(videos::is_ai_labeled.eq(is_ai))
                .execute(&mut conn)?;
            Ok(is_ai)
        }
    }).await.unwrap_or(Ok(false)).unwrap_or(false);

    // 6. Regenerate preview thumbnail for the new start of video
    if let Err(e) = crate::services::previews::generate_previews(
        &video_id,
        &state.seafile,
        &seafile_path,
        std::path::Path::new(&state.previews_dir),
        &state.db,
        state.server_port,
    ).await {
        tracing::error!("Failed to regenerate preview for trimmed video {}: {:?}", video_id, e);
    } else {
        let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoPreview {
            video_id: video_id.clone(),
            preview_url: format!("/api/videos/{}/previews/0", video_id),
        });
    }

    // Send AI labeled status update
    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoAiLabeled {
        video_id: video_id.clone(),
        is_ai_labeled: is_ai,
        is_analyzing: false,
        is_queued: false,
        has_transcript: Some(has_transcript),
    });

    Ok(())
}

pub async fn get_trim_impact(
    State(state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    Path(video_id): Path<String>,
    Query(payload): Query<TrimVideoPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    
    let db = state.db.clone();
    
    let start_ms = (payload.start_sec * 1000.0).round() as i32;
    let end_ms = (payload.end_sec * 1000.0).round() as i32;
    
    let impact = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comments};
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;
        
        let deleted_bouts = bouts::table
            .filter(bouts::video_id.eq(&video_id))
            .filter(bouts::time_start_ms.lt(start_ms).or(bouts::time_end_ms.gt(end_ms)))
            .count()
            .get_result::<i64>(&mut conn)? as i32;

        let deleted_comments = comments::table
            .filter(comments::video_id.eq(&video_id))
            .filter(comments::timestamp_ms.lt(start_ms).or(comments::timestamp_ms.gt(end_ms)))
            .count()
            .get_result::<i64>(&mut conn)? as i32;
            
        Ok::<_, diesel::result::Error>((deleted_bouts, deleted_comments))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(axum::Json(serde_json::json!({
        "deleted_bouts": impact.0,
        "deleted_comments": impact.1
    })))
}

// ── Adaptive Video Optimization ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct OptimizationCandidatesQuery {
    pub older_than_days: Option<i64>,
}

#[derive(Deserialize)]
pub struct BatchOptimizePayload {
    pub video_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct OptimizationCandidateDto {
    pub id: String,
    pub date: String,
    pub seafile_path: String,
    pub duration_ms: Option<i32>,
    pub bouts_count: usize,
    pub fighter_a_name: Option<String>,
    pub fighter_b_name: Option<String>,
}

pub async fn get_optimization_candidates(
    State(state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    Query(params): Query<OptimizationCandidatesQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let candidates = tokio::task::spawn_blocking(move || -> Result<Vec<OptimizationCandidateDto>, diesel::result::Error> {
        use crate::db::schema::{bouts, users, videos};
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;

        let all_videos: Vec<Video> = videos::table
            .filter(videos::is_optimized.eq(false))
            .filter(videos::is_optimizing.eq(false))
            .order(videos::date.desc())
            .load(&mut conn)?;

        let all_bouts: Vec<crate::db::models::Bout> = bouts::table.load(&mut conn)?;
        let mut bouts_by_video: HashMap<String, Vec<crate::db::models::Bout>> = HashMap::new();
        for b in all_bouts {
            bouts_by_video.entry(b.video_id.clone()).or_default().push(b);
        }

        let all_users: Vec<User> = users::table.load(&mut conn)?;
        let users_by_id: HashMap<String, User> = all_users.into_iter().map(|u| (u.id.clone(), u)).collect();

        let today = chrono::Utc::now().naive_utc().date();
        let cutoff_date = params.older_than_days.map(|d| today - chrono::Duration::days(d));

        let mut result = Vec::new();
        for v in all_videos {
            if let Some(cutoff) = cutoff_date {
                if v.date > cutoff {
                    continue;
                }
            }

            let video_bouts = bouts_by_video.get(&v.id).map(|b| b.as_slice()).unwrap_or(&[]);
            // Eligible: at least 1 bout and ALL bouts are human (is_ai == false)
            let is_eligible = !video_bouts.is_empty() && video_bouts.iter().all(|b| !b.is_ai);
            if !is_eligible {
                continue;
            }

            let fighter_a_name = v.fighter_a_id.as_ref().and_then(|id| users_by_id.get(id)).map(|u| u.display_name.clone());
            let fighter_b_name = v.fighter_b_id.as_ref().and_then(|id| users_by_id.get(id)).map(|u| u.display_name.clone());

            result.push(OptimizationCandidateDto {
                id: v.id,
                date: v.date.format("%Y-%m-%d").to_string(),
                seafile_path: v.seafile_path,
                duration_ms: v.duration_ms,
                bouts_count: video_bouts.len(),
                fighter_a_name,
                fighter_b_name,
            });
        }

        Ok(result)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(axum::Json(candidates))
}

pub async fn optimize_video(
    State(state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    Path(video_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let vid = video_id.clone();

    // Verify video and eligibility
    let video = tokio::task::spawn_blocking({
        let db = db.clone();
        let vid = vid.clone();
        move || -> Result<Video, AppError> {
            use crate::db::schema::{bouts, videos};
            use diesel::prelude::*;
            let mut conn = db.get().map_err(|_| AppError::Internal("DB connection error".into()))?;

            let v = videos::table
                .filter(videos::id.eq(&vid))
                .first::<Video>(&mut conn)
                .map_err(|_| AppError::NotFound)?;

            if v.is_optimizing {
                return Err(AppError::BadRequest("Видео уже находится в процессе оптимизации".into()));
            }

            let video_bouts: Vec<crate::db::models::Bout> = bouts::table
                .filter(bouts::video_id.eq(&vid))
                .load(&mut conn)
                .map_err(|_| AppError::Internal("Failed to load bouts".into()))?;

            let is_eligible = !video_bouts.is_empty() && video_bouts.iter().all(|b| !b.is_ai);
            if !is_eligible {
                return Err(AppError::BadRequest("Видео не полностью размечено человеком или не имеет сходов".into()));
            }

            // Set is_optimizing = true
            diesel::update(videos::table.filter(videos::id.eq(&vid)))
                .set(videos::is_optimizing.eq(true))
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            Ok(v)
        }
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Broadcast WS event that optimization has started
    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoOptimized {
        video_id: video_id.clone(),
        is_optimized: video.is_optimized,
        is_optimizing: true,
    });

    // Spawn background task
    let state_clone = state.clone();
    let vid_task = video_id.clone();
    tokio::spawn(async move {
        if let Err(e) = run_optimize_task(state_clone, vid_task.clone()).await {
            tracing::error!("Optimization task failed for video {}: {:?}", vid_task, e);
        }
    });

    Ok(axum::response::Json(serde_json::json!({
        "status": "optimizing",
        "video_id": video_id
    })))
}

pub async fn batch_optimize_videos(
    State(state): State<AppState>,
    crate::middleware::auth::CurrentUser(user): crate::middleware::auth::CurrentUser,
    axum::Json(payload): axum::Json<BatchOptimizePayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let video_ids = payload.video_ids;
    if video_ids.is_empty() {
        return Ok(axum::response::Json(serde_json::json!({ "queued_count": 0 })));
    }

    let state_clone = state.clone();
    tokio::spawn(async move {
        for vid in video_ids {
            // Set is_optimizing in DB
            let db = state_clone.db.clone();
            let vid_c = vid.clone();
            let should_run = tokio::task::spawn_blocking(move || -> bool {
                use crate::db::schema::videos;
                use diesel::prelude::*;
                if let Ok(mut conn) = db.get() {
                    let v = videos::table.filter(videos::id.eq(&vid_c)).first::<Video>(&mut conn);
                    if let Ok(v) = v {
                        if !v.is_optimizing && !v.is_optimized {
                            let _ = diesel::update(videos::table.filter(videos::id.eq(&vid_c)))
                                .set(videos::is_optimizing.eq(true))
                                .execute(&mut conn);
                            return true;
                        }
                    }
                }
                false
            }).await.unwrap_or(false);

            if should_run {
                let _ = state_clone.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoOptimized {
                    video_id: vid.clone(),
                    is_optimized: false,
                    is_optimizing: true,
                });

                if let Err(e) = run_optimize_task(state_clone.clone(), vid.clone()).await {
                    tracing::error!("Batch optimization failed for {}: {:?}", vid, e);
                }
            }
        }
    });

    Ok(axum::response::Json(serde_json::json!({ "status": "queued" })))
}

async fn run_optimize_task(state: AppState, video_id: String) -> Result<(), anyhow::Error> {
    let db = state.db.clone();

    // 1. Get video
    let video = tokio::task::spawn_blocking({
        let vid = video_id.clone();
        let db = db.clone();
        move || {
            use crate::db::schema::videos;
            use diesel::prelude::*;
            let mut conn = db.get().map_err(|_| diesel::result::Error::NotFound)?;
            videos::table
                .filter(videos::id.eq(&vid))
                .first::<Video>(&mut conn)
        }
    })
    .await??;

    let seafile_path = video.seafile_path.clone();
    let temp_dir = get_temp_dir().await;
    let temp_input = format!("{}/opt_in_{}.mp4", temp_dir, video_id);
    let temp_output = format!("{}/opt_out_{}.mp4", temp_dir, video_id);

    // 2. Download original video
    tracing::info!("Optimize: downloading video {} from Seafile to {}...", video_id, temp_input);
    let download_url = state.seafile.get_download_url(&seafile_path).await?;
    let response = reqwest::get(&download_url).await?;
    {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&temp_input).await?;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }
    }

    // 3. Find python binary and script
    let script_candidates = [
        "execution/compress_adaptive_vfr.py",
        "../execution/compress_adaptive_vfr.py",
        "/app/execution/compress_adaptive_vfr.py",
    ];
    let mut script_path = "execution/compress_adaptive_vfr.py".to_string();
    for c in script_candidates {
        if std::path::Path::new(c).exists() {
            script_path = c.to_string();
            break;
        }
    }

    let python_bin = std::env::var("PYTHON_BIN").unwrap_or_else(|_| {
        if std::path::Path::new("venv/Scripts/python.exe").exists() {
            "venv/Scripts/python.exe".to_string()
        } else if std::path::Path::new("venv/bin/python").exists() {
            "venv/bin/python".to_string()
        } else {
            "python".to_string()
        }
    });

    tracing::info!("Optimize: running {} {} on {}...", python_bin, script_path, video_id);

    // Try GPU hevc_nvenc first, fallback to libx265 if needed
    let output = tokio::process::Command::new(&python_bin)
        .arg(&script_path)
        .arg(&temp_input)
        .arg("--video-id")
        .arg(&video_id)
        .arg("-o")
        .arg(&temp_output)
        .arg("--encoder")
        .arg("hevc_nvenc")
        .output()
        .await;

    let success = match output {
        Ok(ref o) if o.status.success() => true,
        _ => {
            tracing::warn!("NVENC encoding failed or unavailable, retrying with libx265 CPU...");
            let fallback_out = tokio::process::Command::new(&python_bin)
                .arg(&script_path)
                .arg(&temp_input)
                .arg("--video-id")
                .arg(&video_id)
                .arg("-o")
                .arg(&temp_output)
                .arg("--encoder")
                .arg("libx265")
                .output()
                .await;
            matches!(fallback_out, Ok(ref o) if o.status.success())
        }
    };

    let _ = tokio::fs::remove_file(&temp_input).await;

    if !success || !std::path::Path::new(&temp_output).exists() {
        let _ = tokio::fs::remove_file(&temp_output).await;
        // Reset is_optimizing in DB
        let vid_clone = video_id.clone();
        let db_clone = db.clone();
        let _ = tokio::task::spawn_blocking(move || {
            use crate::db::schema::videos;
            use diesel::prelude::*;
            if let Ok(mut conn) = db_clone.get() {
                let _ = diesel::update(videos::table.filter(videos::id.eq(&vid_clone)))
                    .set(videos::is_optimizing.eq(false))
                    .execute(&mut conn);
            }
        }).await;

        let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoOptimized {
            video_id: video_id.clone(),
            is_optimized: false,
            is_optimizing: false,
        });

        return Err(anyhow::anyhow!("Adaptive VFR compression failed for {}", video_id));
    }

    // 4. Upload compressed file back to Seafile
    tracing::info!("Optimize: compression complete, uploading back to Seafile: {}...", seafile_path);
    let file_bytes = tokio::fs::read(&temp_output).await?;
    state.seafile.upload_file(&seafile_path, file_bytes).await?;
    let _ = tokio::fs::remove_file(&temp_output).await;

    // 5. Update DB: is_optimized = true, is_optimizing = false
    let vid_clone = video_id.clone();
    let db_clone = db.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        use diesel::prelude::*;
        if let Ok(mut conn) = db_clone.get() {
            let _ = diesel::update(videos::table.filter(videos::id.eq(&vid_clone)))
                .set((
                    videos::is_optimized.eq(true),
                    videos::is_optimizing.eq(false),
                ))
                .execute(&mut conn);
        }
    }).await?;

    // 6. Broadcast WS update
    let _ = state.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoOptimized {
        video_id: video_id.clone(),
        is_optimized: true,
        is_optimizing: false,
    });

    tracing::info!("Optimize: video {} successfully optimized and updated!", video_id);
    Ok(())
}


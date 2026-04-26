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
    db::models::{Bout, Comment, User, Video},
    errors::AppError,
    middleware::auth::CurrentUser,
    state::AppState,
};

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
}

#[derive(Serialize)]
pub struct CommentAuthorDto {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
}

#[derive(Serialize)]
pub struct CommentDto {
    pub id: i32,
    pub author: CommentAuthorDto,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct VideoFullDto {
    pub id: String,
    pub date: String,
    pub fighter_a: Option<VideoFighterDto>,
    pub fighter_b: Option<VideoFighterDto>,
    pub stream_url: String,
    pub duration_ms: Option<i32>,
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
    }
}

fn build_video_full(
    video: &Video,
    bouts: Vec<Bout>,
    comments: Vec<Comment>,
    users_map: &HashMap<String, User>,
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
            let author = users_map
                .get(&c.author_id)
                .map(|u| CommentAuthorDto {
                    id: u.id.clone(),
                    display_name: u.display_name.clone(),
                    avatar_url: format!("/api/users/{}/avatar", u.id),
                })
                .unwrap_or_else(|| CommentAuthorDto {
                    id: c.author_id.clone(),
                    display_name: "Unknown".to_string(),
                    avatar_url: format!("/api/users/{}/avatar", c.author_id),
                });
            CommentDto {
                id: c.id,
                author,
                timestamp_ms: c.timestamp_ms,
                text: c.text.clone(),
                reply_to_id: c.reply_to_id,
                created_at: c.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
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

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_videos(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(params): Query<VideoListQuery>,
) -> Result<Json<Vec<VideoListDto>>, AppError> {
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
                }
            })
            .collect();

        Ok(dtos)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn get_video(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(video_id): Path<String>,
) -> Result<Json<VideoFullDto>, AppError> {
    let db = state.db.clone();

    let (dto, seafile_path) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comments, videos};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let video = videos::table
            .filter(videos::id.eq(&video_id))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let seafile_path = video.seafile_path.clone();

        let video_bouts = bouts::table
            .filter(bouts::video_id.eq(&video_id))
            .order(bouts::order_index.asc())
            .load::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let video_comments = comments::table
            .filter(comments::video_id.eq(&video_id))
            .order(comments::id.asc())
            .load::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let users_map = load_users_for_video(&video, &video_comments, &mut conn)?;

        let dto = build_video_full(
            &video,
            video_bouts,
            video_comments,
            &users_map,
            String::new(),
        );

        Ok::<_, AppError>((dto, seafile_path))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let stream_url = format!("/api/videos/{}/stream", dto.id);

    Ok(Json(VideoFullDto { stream_url, ..dto }))
}

pub async fn patch_video(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(video_id): Path<String>,
    Json(body): Json<PatchVideoRequest>,
) -> Result<Json<VideoFullDto>, AppError> {
    let db = state.db.clone();

    let (dto, seafile_path) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comments, videos};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let rows = diesel::update(videos::table.filter(videos::id.eq(&video_id)))
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
            .filter(videos::id.eq(&video_id))
            .first::<Video>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let seafile_path = video.seafile_path.clone();

        let video_bouts = bouts::table
            .filter(bouts::video_id.eq(&video_id))
            .order(bouts::order_index.asc())
            .load::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let video_comments = comments::table
            .filter(comments::video_id.eq(&video_id))
            .order(comments::id.asc())
            .load::<Comment>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let users_map = load_users_for_video(&video, &video_comments, &mut conn)?;

        let dto = build_video_full(
            &video,
            video_bouts,
            video_comments,
            &users_map,
            String::new(),
        );

        Ok::<_, AppError>((dto, seafile_path))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let stream_url = format!("/api/videos/{}/stream", dto.id);

    Ok(Json(VideoFullDto { stream_url, ..dto }))
}

// ── Seafile previews ──────────────────────────────────────────────────────────

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

        tokio::spawn(async move {
            match seafile.get_download_url(&seafile_path).await {
                Ok(url) => {
                    if let Err(e) = crate::previews::generate_previews(
                        &vid_id,
                        &url,
                        std::path::Path::new(&previews_dir),
                        &db,
                    )
                    .await
                    {
                        tracing::error!("generate_previews failed for {vid_id}: {e:?}");
                    }
                }
                Err(e) => tracing::error!("get_download_url failed for {vid_id}: {e}"),
            }
        });

        return Ok(StatusCode::ACCEPTED.into_response());
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

    let seafile_path = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let video = videos::table
            .filter(videos::id.eq(&video_id))
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

    let seafile_resp = state
        .seafile
        .fetch_range(&seafile_path, range.as_deref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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

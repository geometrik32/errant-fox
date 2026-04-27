use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    db::models::{Bout, NewUser, Technique, User, Video},
    errors::AppError,
    middleware::auth::CurrentUser,
    state::AppState,
};

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct FighterDto {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
    pub is_admin: bool,
}

#[derive(Serialize)]
pub struct FighterBoutDto {
    pub id: i32,
    pub video_id: String,
    pub video_date: String,
    pub opponent_id: String,
    pub opponent_name: String,
    pub order_index: i32,
    pub time_start_ms: i32,
    pub time_end_ms: i32,
    pub my_score: i32,
    pub opponent_score: i32,
    pub my_technique_id: Option<i32>,
    pub my_technique_name: Option<String>,
    pub my_hit_zone: Option<String>,
    pub my_result: Option<String>,
    pub opponent_technique_id: Option<i32>,
    pub opponent_technique_name: Option<String>,
    pub opponent_hit_zone: Option<String>,
    pub opponent_result: Option<String>,
}

#[derive(Serialize)]
pub struct UserMeDto {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub is_admin: bool,
    pub avatar_url: String,
    pub color: Option<String>,
}

fn to_fighter_dto(u: &User) -> FighterDto {
    FighterDto {
        id: u.id.clone(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        avatar_url: format!("/api/users/{}/avatar", u.id),
        color: u.color.clone(),
        is_admin: u.is_admin,
    }
}

fn to_me_dto(u: &User) -> UserMeDto {
    UserMeDto {
        id: u.id.clone(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        is_admin: u.is_admin,
        avatar_url: format!("/api/users/{}/avatar", u.id),
        color: u.color.clone(),
    }
}

// ── Request bodies ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PatchMeRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub is_admin: bool,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct PatchAdminUserRequest {
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub color: Option<String>,
    pub is_admin: Option<bool>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_fighters(
    State(state): State<AppState>,
    _user: CurrentUser,
) -> Result<Json<Vec<FighterDto>>, AppError> {
    let db = state.db.clone();
    let fighters = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::users;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        users
            .load::<User>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(fighters.iter().map(to_fighter_dto).collect()))
}

pub async fn fighter_bouts(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(fighter_id): Path<String>,
) -> Result<Json<Vec<FighterBoutDto>>, AppError> {
    let db = state.db.clone();

    let result = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, techniques, users, videos};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let video_list = videos::table
            .filter(
                videos::fighter_a_id
                    .eq(&fighter_id)
                    .or(videos::fighter_b_id.eq(&fighter_id)),
            )
            .load::<Video>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if video_list.is_empty() {
            return Ok(vec![]);
        }

        let video_ids: Vec<String> = video_list.iter().map(|v| v.id.clone()).collect();

        let all_bouts = bouts::table
            .filter(bouts::video_id.eq_any(&video_ids))
            .order((bouts::video_id.asc(), bouts::order_index.asc()))
            .load::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let all_techniques = techniques::table
            .load::<Technique>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let technique_map: HashMap<i32, String> =
            all_techniques.into_iter().map(|t| (t.id, t.name)).collect();

        let opponent_ids: Vec<String> = video_list
            .iter()
            .filter_map(|v| {
                if v.fighter_a_id.as_deref() == Some(&fighter_id) {
                    v.fighter_b_id.clone()
                } else {
                    v.fighter_a_id.clone()
                }
            })
            .collect();

        let opponent_users = if opponent_ids.is_empty() {
            vec![]
        } else {
            users::table
                .filter(users::id.eq_any(&opponent_ids))
                .load::<User>(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?
        };
        let opponent_map: HashMap<String, User> =
            opponent_users.into_iter().map(|u| (u.id.clone(), u)).collect();

        let video_map: HashMap<&str, &Video> =
            video_list.iter().map(|v| (v.id.as_str(), v)).collect();

        let mut dtos = Vec::new();
        for bout in &all_bouts {
            let video = match video_map.get(bout.video_id.as_str()) {
                Some(v) => v,
                None => continue,
            };

            let am_a = video.fighter_a_id.as_deref() == Some(&fighter_id);
            let opponent_id = if am_a {
                video.fighter_b_id.clone().unwrap_or_default()
            } else {
                video.fighter_a_id.clone().unwrap_or_default()
            };
            let opponent_name = opponent_map
                .get(&opponent_id)
                .map(|u| u.display_name.clone())
                .unwrap_or_default();

            let my_technique_id = if am_a { bout.technique_a_id } else { bout.technique_b_id };
            let opp_technique_id = if am_a { bout.technique_b_id } else { bout.technique_a_id };

            dtos.push(FighterBoutDto {
                id: bout.id,
                video_id: bout.video_id.clone(),
                video_date: video.date.format("%Y-%m-%d").to_string(),
                opponent_id,
                opponent_name,
                order_index: bout.order_index,
                time_start_ms: bout.time_start_ms,
                time_end_ms: bout.time_end_ms,
                my_score: if am_a { bout.score_a } else { bout.score_b },
                opponent_score: if am_a { bout.score_b } else { bout.score_a },
                my_technique_id,
                my_technique_name: my_technique_id.and_then(|id| technique_map.get(&id).cloned()),
                my_hit_zone: if am_a { bout.hit_zone_a.clone() } else { bout.hit_zone_b.clone() },
                my_result: if am_a { bout.result_a.clone() } else { bout.result_b.clone() },
                opponent_technique_id: opp_technique_id,
                opponent_technique_name: opp_technique_id
                    .and_then(|id| technique_map.get(&id).cloned()),
                opponent_hit_zone: if am_a {
                    bout.hit_zone_b.clone()
                } else {
                    bout.hit_zone_a.clone()
                },
                opponent_result: if am_a { bout.result_b.clone() } else { bout.result_a.clone() },
            });
        }

        Ok::<Vec<FighterBoutDto>, AppError>(dtos)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn patch_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<PatchMeRequest>,
) -> Result<Json<UserMeDto>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

    let updated = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{color, display_name, id, password_hash, username, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let new_username = body.username.unwrap_or(user.username);
        let new_name = body.display_name.unwrap_or(user.display_name);
        let new_hash = if let Some(pw) = body.password {
            bcrypt::hash(&pw, bcrypt::DEFAULT_COST)
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            user.password_hash
        };
        let new_color = body.color.or(user.color);

        diesel::update(users.filter(id.eq(&user_id)))
            .set((
                username.eq(&new_username),
                display_name.eq(&new_name),
                password_hash.eq(&new_hash),
                color.eq(&new_color),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        users
            .filter(id.eq(&user_id))
            .first::<User>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(to_me_dto(&updated)))
}

pub async fn upload_avatar(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let avatars_dir = state.avatars_dir.clone();
    let user_id = user.id.clone();

    let mut file_data: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name().unwrap_or("") == "avatar" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            if data.len() > 2 * 1024 * 1024 {
                return Err(AppError::BadRequest("Avatar too large (max 2 MB)".to_string()));
            }
            file_data = Some(data.to_vec());
        }
    }

    let data = file_data
        .ok_or_else(|| AppError::BadRequest("Missing 'avatar' field in form".to_string()))?;

    let path = format!("{}/{}.jpg", avatars_dir, user_id);
    tokio::fs::write(&path, data)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "avatar_url": format!("/api/users/{}/avatar", user_id) }),
    ))
}

pub async fn get_avatar(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Response, AppError> {
    let path = format!("{}/{}.jpg", state.avatars_dir, user_id);
    match tokio::fs::read(&path).await {
        Ok(data) => Ok((
            [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
            data,
        )
            .into_response()),
        Err(_) => Err(AppError::NotFound),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserMeDto>), AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let new_id = Uuid::new_v4().to_string();
    let new_id_clone = new_id.clone();

    let created = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{id, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let new_user = NewUser {
            id: new_id_clone.clone(),
            username: body.username,
            display_name: body.display_name,
            password_hash,
            is_admin: body.is_admin,
            avatar_path: None,
            color: body.color,
        };

        diesel::insert_into(users)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        users
            .filter(id.eq(&new_id_clone))
            .first::<User>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok((StatusCode::CREATED, Json(to_me_dto(&created))))
}

pub async fn patch_admin_user(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(user_id): Path<String>,
    Json(body): Json<PatchAdminUserRequest>,
) -> Result<Json<UserMeDto>, AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();

    let updated = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{color, display_name, id, is_admin, password_hash, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let target_user = users
            .filter(id.eq(&user_id))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let new_name = body.display_name.unwrap_or(target_user.display_name);
        let new_hash = if let Some(pw) = body.password {
            bcrypt::hash(&pw, bcrypt::DEFAULT_COST)
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            target_user.password_hash
        };
        let new_color = body.color.or(target_user.color);
        let new_is_admin = body.is_admin.unwrap_or(target_user.is_admin);

        diesel::update(users.filter(id.eq(&user_id)))
            .set((
                display_name.eq(&new_name),
                password_hash.eq(&new_hash),
                color.eq(&new_color),
                is_admin.eq(new_is_admin),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        users
            .filter(id.eq(&user_id))
            .first::<User>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(to_me_dto(&updated)))
}

pub async fn upload_avatar_for(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(user_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let avatars_dir = state.avatars_dir.clone();

    let mut file_data: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name().unwrap_or("") == "avatar" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            if data.len() > 2 * 1024 * 1024 {
                return Err(AppError::BadRequest("Avatar too large (max 2 MB)".to_string()));
            }
            file_data = Some(data.to_vec());
        }
    }

    let data = file_data
        .ok_or_else(|| AppError::BadRequest("Missing 'avatar' field in form".to_string()))?;

    let path = format!("{}/{}.jpg", avatars_dir, user_id);
    tokio::fs::write(&path, data)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "avatar_url": format!("/api/users/{}/avatar", user_id) }),
    ))
}

pub async fn delete_user(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }
    if current.id == user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    let db = state.db.clone();

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{id, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let deleted = diesel::delete(users.filter(id.eq(&user_id)))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if deleted == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "ok": true })))
}

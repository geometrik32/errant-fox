use axum::{extract::State, Json};
use bcrypt;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{db::models::User, errors::AppError, middleware::auth::CurrentUser, state::AppState};

fn generate_color(user_id: &str) -> String {
    const PALETTE: &[&str] = &[
        "#e05252", "#DB841F", "#d4c017", "#6aaa5e",
        "#4a9e8a", "#4a8eaa", "#5272e0", "#8052e0",
        "#aa52e0", "#e052aa", "#e07252", "#52aae0",
    ];
    let hash: usize = user_id.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize));
    PALETTE[hash % PALETTE.len()].to_string()
}

// ── JWT ───────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn make_token(user_id: &str, secret: &str) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims { sub: user_id.to_string(), exp };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized("Unauthorized".to_string()))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub is_admin: bool,
    pub avatar_url: String,
    pub color: String,
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

fn to_user_dto(u: &User) -> UserDto {
    UserDto {
        id: u.id.clone(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        is_admin: u.is_admin,
        avatar_url: format!("/api/users/{}/avatar", u.id),
        color: u.color.clone().unwrap_or_else(|| generate_color(&u.id)),
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

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserDto,
}

#[derive(Deserialize)]
pub struct PatchMeRequest {
    pub display_name: Option<String>,
    pub password: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let db = state.db.clone();
    let username_input = body.username.clone();
    let password_input = body.password.clone();

    let user = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{color as color_col, id, username, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let maybe: Option<User> = users
            .filter(username.eq(&username_input))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let user = maybe
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        let valid = bcrypt::verify(&password_input, &user.password_hash)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !valid {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        // Ensure color is assigned
        let user = if user.color.is_none() {
            let new_color = generate_color(&user.id);
            diesel::update(users.filter(id.eq(&user.id)))
                .set(color_col.eq(&new_color))
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            User { color: Some(new_color), ..user }
        } else {
            user
        };

        Ok(user)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let token = make_token(&user.id, &state.jwt_secret)?;

    Ok(Json(LoginResponse {
        token,
        user: to_user_dto(&user),
    }))
}

pub async fn get_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<UserMeDto>, AppError> {
    if user.color.is_some() {
        return Ok(Json(to_me_dto(&user)));
    }

    let new_color = generate_color(&user.id);
    let user_id = user.id.clone();
    let color_clone = new_color.clone();
    let db = state.db.clone();

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{color as color_col, id, users};
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(users.filter(id.eq(&user_id)))
            .set(color_col.eq(&color_clone))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(to_me_dto(&User { color: Some(new_color), ..user })))
}

pub async fn patch_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<PatchMeRequest>,
) -> Result<Json<UserMeDto>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

    let updated = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{display_name, id, password_hash, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let new_name = body.display_name.unwrap_or(user.display_name);
        let new_hash = if let Some(pw) = body.password {
            bcrypt::hash(&pw, bcrypt::DEFAULT_COST)
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            user.password_hash
        };

        diesel::update(users.filter(id.eq(&user_id)))
            .set((display_name.eq(&new_name), password_hash.eq(&new_hash)))
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

use axum::{extract::State, Json};
use bcrypt;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{db::models::User, errors::AppError, middleware::auth::CurrentUser, state::AppState};

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
        use crate::db::schema::users::dsl::{username, users};

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

pub async fn get_me(CurrentUser(user): CurrentUser) -> Result<Json<UserMeDto>, AppError> {
    Ok(Json(to_me_dto(&user)))
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

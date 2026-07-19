use axum::{extract::State, Json};
use bcrypt;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{db::models::User, errors::AppError, middleware::auth::CurrentUser, state::AppState};

pub fn generate_color(user_id: &str) -> String {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShareClaims {
    pub video_id: String,
    pub bout_id: Option<i32>,
    pub exp: usize,
}

pub fn make_share_token(video_id: &str, bout_id: Option<i32>, secret: &str) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(365 * 100)).timestamp() as usize;
    let claims = ShareClaims {
        video_id: video_id.to_string(),
        bout_id,
        exp,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
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
    pub role: String,
}

#[derive(Serialize)]
pub struct UserMeDto {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub is_admin: bool,
    pub avatar_url: String,
    pub color: Option<String>,
    pub vk_id: Option<String>,
    pub role: String,
}

fn to_user_dto(u: &User) -> UserDto {
    UserDto {
        id: u.id.clone(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        is_admin: u.is_admin,
        avatar_url: format!("/api/users/{}/avatar", u.id),
        color: u.color.clone().unwrap_or_else(|| generate_color(&u.id)),
        role: u.role.clone(),
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
        vk_id: u.vk_id.clone(),
        role: u.role.clone(),
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

#[derive(Deserialize)]
pub struct VkLoginRequest {
    pub code: String,
    pub redirect_uri: String,
    pub code_verifier: String,
    pub device_id: String,
}

#[derive(Serialize)]
pub struct VkConfigResponse {
    pub client_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct VkTokenResponse {
    access_token: String,
    user_id: i64,
}

#[derive(Deserialize, Debug)]
struct VkUserInfo {
    user_id: String,
    first_name: String,
    last_name: String,
    avatar: Option<String>,
}

#[derive(Deserialize, Debug)]
struct VkUserInfoResponse {
    user: VkUserInfo,
}

pub async fn vk_config(State(state): State<AppState>) -> Json<VkConfigResponse> {
    Json(VkConfigResponse {
        client_id: state.vk_app_id.clone(),
    })
}

pub async fn vk_login(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<VkLoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let app_id = state
        .vk_app_id
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("VK Auth is not configured on server".to_string()))?;
    let app_secret = state
        .vk_app_secret
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("VK Auth is not configured on server".to_string()))?;

    let client = reqwest::Client::new();

    // 0. Check if request comes from already authenticated user to perform linking
    let mut current_user_opt: Option<User> = None;
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Ok(claims) = verify_token(token, &state.jwt_secret) {
                let db = state.db.clone();
                let user_id = claims.sub.clone();
                if let Ok(Ok(Some(u))) = tokio::task::spawn_blocking(move || {
                    use crate::db::schema::users::dsl::{id, users};
                    use diesel::prelude::*;
                    let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
                    users.filter(id.eq(&user_id)).first::<User>(&mut conn).optional().map_err(|e| AppError::Internal(e.to_string()))
                }).await {
                    current_user_opt = Some(u);
                }
            }
        }
    }

    // 1. Exchange code for access_token and user_id via POST to https://id.vk.com/oauth2/auth
    let response = client
        .post("https://id.vk.com/oauth2/auth")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", app_id.as_str()),
            ("client_secret", app_secret.as_str()),
            ("code", body.code.as_str()),
            ("redirect_uri", body.redirect_uri.as_str()),
            ("code_verifier", body.code_verifier.as_str()),
            ("device_id", body.device_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to VK OAuth: {}", e)))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read VK OAuth response text: {}", e)))?;

    tracing::info!("VK OAuth response (status {}): {}", status, response_text);

    if !status.is_success() {
        return Err(AppError::Unauthorized(format!("VK OAuth error (status {}): {}", status, response_text)));
    }

    let token_resp = serde_json::from_str::<VkTokenResponse>(&response_text)
        .map_err(|e| AppError::Unauthorized(format!("VK OAuth parse error (response: {}): {}", response_text, e)))?;

    // 2. Fetch VK user profile via POST to https://id.vk.com/oauth2/user_info
    let profile_response = client
        .post("https://id.vk.com/oauth2/user_info")
        .bearer_auth(&token_resp.access_token)
        .json(&serde_json::json!({
            "client_id": app_id.as_str()
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to VK API: {}", e)))?;

    let profile_status = profile_response.status();
    let profile_text = profile_response
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read VK user_info response text: {}", e)))?;

    tracing::info!("VK user_info response (status {}): {}", profile_status, profile_text);

    if !profile_status.is_success() {
        return Err(AppError::Unauthorized(format!("VK user_info error (status {}): {}", profile_status, profile_text)));
    }

    let profile_resp = serde_json::from_str::<VkUserInfoResponse>(&profile_text)
        .map_err(|e| AppError::Internal(format!("Failed to parse VK user_info response: {}", e)))?;

    let vk_user = profile_resp.user;

    let vk_id_str = vk_user.user_id.clone();
    let display_name_val = format!("{} {}", vk_user.first_name, vk_user.last_name);
    let avatar_url_opt = vk_user.avatar.clone();

    let db = state.db.clone();
    let user_id_for_avatar = vk_id_str.clone();

    // 3. Database lookup and user insert/update/link
    let user = if let Some(current_user) = current_user_opt {
        // Link VK ID to the logged-in user
        let is_already_bound = tokio::task::spawn_blocking({
            let db = db.clone();
            let vk_id_str = vk_id_str.clone();
            let current_user_id = current_user.id.clone();
            move || {
                use crate::db::schema::users::dsl::{id, users, vk_id};
                use diesel::prelude::*;
                let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
                let existing: Option<User> = users
                    .filter(vk_id.eq(&vk_id_str))
                    .filter(id.ne(&current_user_id))
                    .first::<User>(&mut conn)
                    .optional()
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                Ok::<bool, AppError>(existing.is_some())
            }
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

        if is_already_bound {
            return Err(AppError::BadRequest("Этот аккаунт ВКонтакте уже привязан к другому пользователю".to_string()));
        }

        tokio::task::spawn_blocking({
            let db = db.clone();
            let vk_id_str = vk_id_str.clone();
            let current_user_id = current_user.id.clone();
            move || {
                use crate::db::schema::users::dsl::{id, users, vk_id};
                use diesel::prelude::*;
                let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
                diesel::update(users.filter(id.eq(&current_user_id)))
                    .set(vk_id.eq(Some(&vk_id_str)))
                    .execute(&mut conn)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                users.filter(id.eq(&current_user_id)).first::<User>(&mut conn).map_err(|e| AppError::Internal(e.to_string()))
            }
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??
    } else {
        // Standard VK Login / Registration
        tokio::task::spawn_blocking(move || {
            use crate::db::schema::users::dsl::{display_name as name_col, id, users, vk_id};
            use diesel::prelude::*;

            let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

            // Try to find user by vk_id
            let existing: Option<User> = users
                .filter(vk_id.eq(&user_id_for_avatar))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some(u) = existing {
                Ok(u)
            } else {
                // Create a new guest user
                let new_id = format!("vk_{}", user_id_for_avatar);
                let username = format!("vk_{}", user_id_for_avatar);
                let new_color = generate_color(&new_id);

                let new_user = crate::db::models::NewUser {
                    id: new_id.clone(),
                    username,
                    display_name: display_name_val.clone(),
                    password_hash: "".to_string(), // no local password
                    is_admin: false,
                    avatar_path: Some(format!("{}.jpg", new_id)),
                    color: Some(new_color.clone()),
                    vk_id: Some(user_id_for_avatar.clone()),
                    role: "guest".to_string(),
                };

                diesel::insert_into(users)
                    .values(&new_user)
                    .execute(&mut conn)
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                users
                    .filter(id.eq(&new_id))
                    .first::<User>(&mut conn)
                    .map_err(|e| AppError::Internal(e.to_string()))
            }
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??
    };

    // 4. Download avatar in the background if available
    if let Some(avatar_url) = avatar_url_opt {
        let avatars_dir = state.avatars_dir.clone();
        let user_id = user.id.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client.get(&avatar_url).send().await {
                if let Ok(bytes) = resp.bytes().await {
                    let path = format!("{}/{}.jpg", avatars_dir, user_id);
                    if let Err(e) = tokio::fs::write(&path, bytes).await {
                        tracing::error!("Failed to save VK avatar: {}", e);
                    }
                }
            }
        });
    }

    let token = make_token(&user.id, &state.jwt_secret)?;

    Ok(Json(LoginResponse {
        token,
        user: to_user_dto(&user),
    }))
}

pub async fn vk_unlink(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<UserMeDto>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

    let updated = tokio::task::spawn_blocking(move || {
        use crate::db::schema::users::dsl::{id, users, vk_id};
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(users.filter(id.eq(&user_id)))
            .set(vk_id.eq(None::<String>))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        users.filter(id.eq(&user_id)).first::<User>(&mut conn).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(to_me_dto(&updated)))
}

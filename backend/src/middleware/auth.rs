use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};

use crate::{db::models::User, errors::AppError, state::AppState};

pub struct CurrentUser(pub User);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = if let Some(header_val) = auth_header {
            header_val
                .strip_prefix("Bearer ")
                .ok_or_else(|| AppError::Unauthorized("Unauthorized".to_string()))?
                .to_string()
        } else {
            let query = parts.uri.query().unwrap_or("");
            query
                .split('&')
                .find(|p| p.starts_with("token="))
                .map(|p| p.strip_prefix("token=").unwrap_or(""))
                .filter(|t| !t.is_empty())
                .map(|t| t.to_string())
                .ok_or_else(|| AppError::Unauthorized("Unauthorized".to_string()))?
        };

        let claims = crate::api::auth::verify_token(&token, &app_state.jwt_secret)?;
        let user_id = claims.sub;

        let db = app_state.db.clone();
        let user = tokio::task::spawn_blocking(move || {
            use crate::db::schema::users::dsl::{id, users};
            use diesel::prelude::*;

            let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

            users
                .filter(id.eq(&user_id))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or_else(|| AppError::Unauthorized("Unauthorized".to_string()))
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

        Ok(CurrentUser(user))
    }
}

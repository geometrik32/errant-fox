use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden,
    NotFound,
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail) = match self {
            AppError::Unauthorized(msg) => {
                tracing::warn!("unauthorized error: {msg}");
                (StatusCode::UNAUTHORIZED, "Unauthorized", msg)
            }
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden", "Forbidden".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found", "Not found".to_string()),
            AppError::BadRequest(msg) => {
                tracing::warn!("bad request error: {msg}");
                (StatusCode::BAD_REQUEST, "Bad Request", msg)
            }
            AppError::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", "Internal server error".to_string())
            }
        };

        let body = json!({
            "type": "about:blank",
            "title": title,
            "status": status.as_u16(),
            "detail": detail,
            "error": detail // Backward compatibility for existing legacy frontend code
        });

        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}

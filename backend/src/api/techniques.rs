use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::Deserialize;

use crate::{
    db::models::{NewTechnique, Technique},
    errors::AppError,
    middleware::auth::CurrentUser,
    state::AppState,
};

pub async fn list_techniques(
    State(state): State<AppState>,
    _user: CurrentUser,
) -> Result<Json<Vec<Technique>>, AppError> {
    let db = state.db.clone();
    let techniques = tokio::task::spawn_blocking(move || {
        use crate::db::schema::techniques::dsl::techniques;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        techniques
            .load::<Technique>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(techniques))
}

#[derive(Deserialize)]
pub struct CreateTechniqueRequest {
    pub name: String,
}

pub async fn create_technique(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Json(body): Json<CreateTechniqueRequest>,
) -> Result<(StatusCode, Json<Technique>), AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let created = tokio::task::spawn_blocking(move || {
        use crate::db::schema::techniques::dsl::{id, techniques};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        diesel::insert_into(techniques)
            .values(&NewTechnique { name: body.name })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        techniques
            .order(id.desc())
            .first::<Technique>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok((StatusCode::CREATED, Json(created)))
}

#[derive(Deserialize)]
pub struct RenameTechniqueRequest {
    pub name: String,
}

pub async fn rename_technique(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(technique_id): Path<i32>,
    Json(body): Json<RenameTechniqueRequest>,
) -> Result<Json<Technique>, AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let updated = tokio::task::spawn_blocking(move || {
        use crate::db::schema::techniques::dsl::{id, name, techniques};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let rows = diesel::update(techniques.filter(id.eq(technique_id)))
            .set(name.eq(&body.name))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound);
        }

        techniques
            .filter(id.eq(technique_id))
            .first::<Technique>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(updated))
}

pub async fn delete_technique(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(technique_id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::bouts;
        use crate::db::schema::techniques::dsl::{id, techniques};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        // Null out technique references in bouts before deleting
        diesel::update(bouts::table.filter(bouts::technique_a_id.eq(technique_id)))
            .set(bouts::technique_a_id.eq::<Option<i32>>(None))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        diesel::update(bouts::table.filter(bouts::technique_b_id.eq(technique_id)))
            .set(bouts::technique_b_id.eq::<Option<i32>>(None))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let deleted = diesel::delete(techniques.filter(id.eq(technique_id)))
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

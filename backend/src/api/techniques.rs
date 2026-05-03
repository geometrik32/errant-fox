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
    pub description: Option<String>,
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
            .values(&NewTechnique { name: body.name, description: body.description })
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
pub struct PatchTechniqueRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn patch_technique(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(technique_id): Path<i32>,
    Json(body): Json<PatchTechniqueRequest>,
) -> Result<Json<Technique>, AppError> {
    if !current.is_admin {
        return Err(AppError::Forbidden);
    }

    let db = state.db.clone();
    let updated = tokio::task::spawn_blocking(move || {
        use crate::db::schema::techniques::dsl::{description, id, name, techniques};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            techniques.filter(id.eq(technique_id)),
        ))
        .get_result(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if !exists {
            return Err(AppError::NotFound);
        }

        if let Some(new_name) = &body.name {
            diesel::update(techniques.filter(id.eq(technique_id)))
                .set(name.eq(new_name))
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        if let Some(new_desc) = &body.description {
            let desc_val: Option<&str> = if new_desc.is_empty() { None } else { Some(new_desc.as_str()) };
            diesel::update(techniques.filter(id.eq(technique_id)))
                .set(description.eq(desc_val))
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
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

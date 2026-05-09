use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    db::models::{Bout, NewBout},
    errors::AppError,
    middleware::auth::CurrentUser,
    state::AppState,
    services::ws::{WsBout, WsEvent},
};

// ── Response DTO ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct BoutResponse {
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

fn to_response(b: &Bout) -> BoutResponse {
    BoutResponse {
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

fn to_ws_bout(b: &Bout) -> WsBout {
    WsBout {
        id: b.id,
        video_id: b.video_id.clone(),
        order_index: b.order_index,
        time_start_ms: b.time_start_ms,
        time_end_ms: b.time_end_ms,
        score_a: b.score_a,
        score_b: b.score_b,
        technique_a_id: b.technique_a_id,
        technique_b_id: b.technique_b_id,
        hit_zone_a: b.hit_zone_a.clone(),
        hit_zone_b: b.hit_zone_b.clone(),
        result_a: b.result_a.clone(),
        result_b: b.result_b.clone(),
    }
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateBoutRequest {
    pub video_id: String,
    pub time_start_ms: i32,
    pub time_end_ms: i32,
}

// Distinguishes absent field (None) from explicit null (Some(None)) from value (Some(Some(v))).
fn deser_opt_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
pub struct PatchBoutRequest {
    pub time_start_ms: Option<i32>,
    pub time_end_ms: Option<i32>,
    pub score_a: Option<i32>,
    pub score_b: Option<i32>,
    #[serde(default, deserialize_with = "deser_opt_nullable")]
    pub technique_a_id: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deser_opt_nullable")]
    pub technique_b_id: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deser_opt_nullable")]
    pub hit_zone_a: Option<Option<String>>,
    #[serde(default, deserialize_with = "deser_opt_nullable")]
    pub hit_zone_b: Option<Option<String>>,
    #[serde(default, deserialize_with = "deser_opt_nullable")]
    pub result_a: Option<Option<String>>,
    #[serde(default, deserialize_with = "deser_opt_nullable")]
    pub result_b: Option<Option<String>>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn post_bout(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(body): Json<CreateBoutRequest>,
) -> Result<(StatusCode, Json<BoutResponse>), AppError> {
    let db = state.db.clone();

    let bout = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, videos};
        use diesel::dsl::max;

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            videos::table.filter(videos::id.eq(&body.video_id)),
        ))
        .get_result(&mut conn)
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if !exists {
            return Err(AppError::NotFound);
        }

        let max_order: Option<i32> = bouts::table
            .filter(bouts::video_id.eq(&body.video_id))
            .select(max(bouts::order_index))
            .first(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let order_index = max_order.map(|m| m + 1).unwrap_or(1);

        diesel::insert_into(bouts::table)
            .values(&NewBout {
                video_id: body.video_id.clone(),
                order_index,
                time_start_ms: body.time_start_ms,
                time_end_ms: body.time_end_ms,
                score_a: 0,
                score_b: 0,
                technique_a_id: None,
                technique_b_id: None,
                hit_zone_a: None,
                hit_zone_b: None,
                result_a: None,
                result_b: None,
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        bouts::table
            .filter(bouts::video_id.eq(&body.video_id))
            .order(bouts::id.desc())
            .first::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(WsEvent::UpdateBout(to_ws_bout(&bout)));

    Ok((StatusCode::CREATED, Json(to_response(&bout))))
}

pub async fn patch_bout(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<i32>,
    Json(body): Json<PatchBoutRequest>,
) -> Result<Json<BoutResponse>, AppError> {
    let db = state.db.clone();

    let bout = tokio::task::spawn_blocking(move || {
        use crate::db::schema::bouts;

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let cur = bouts::table
            .filter(bouts::id.eq(id))
            .first::<Bout>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let time_start_ms = body.time_start_ms.unwrap_or(cur.time_start_ms);
        let time_end_ms = body.time_end_ms.unwrap_or(cur.time_end_ms);
        let score_a = body.score_a.unwrap_or(cur.score_a);
        let score_b = body.score_b.unwrap_or(cur.score_b);
        let technique_a_id = body.technique_a_id.unwrap_or(cur.technique_a_id);
        let technique_b_id = body.technique_b_id.unwrap_or(cur.technique_b_id);
        let hit_zone_a = body.hit_zone_a.unwrap_or(cur.hit_zone_a);
        let hit_zone_b = body.hit_zone_b.unwrap_or(cur.hit_zone_b);
        let result_a = body.result_a.unwrap_or(cur.result_a);
        let result_b = body.result_b.unwrap_or(cur.result_b);

        diesel::update(bouts::table.filter(bouts::id.eq(id)))
            .set((
                bouts::time_start_ms.eq(time_start_ms),
                bouts::time_end_ms.eq(time_end_ms),
                bouts::score_a.eq(score_a),
                bouts::score_b.eq(score_b),
                bouts::technique_a_id.eq(technique_a_id),
                bouts::technique_b_id.eq(technique_b_id),
                bouts::hit_zone_a.eq(hit_zone_a),
                bouts::hit_zone_b.eq(hit_zone_b),
                bouts::result_a.eq(result_a),
                bouts::result_b.eq(result_b),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        bouts::table
            .filter(bouts::id.eq(id))
            .first::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(WsEvent::UpdateBout(to_ws_bout(&bout)));

    Ok(Json(to_response(&bout)))
}

pub async fn delete_bout(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.db.clone();

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::bouts;

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let deleted = diesel::delete(bouts::table.filter(bouts::id.eq(id)))
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

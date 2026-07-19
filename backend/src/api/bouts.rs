use axum::{
    extract::{Path, State, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    db::models::{Bout, NewBout, Video, User},
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
        is_ai: b.is_ai,
        deleted: None,
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

async fn broadcast_video_score(
    db: crate::db::DbPool,
    ws_hub: crate::services::ws::WsHub,
    video_id: String,
) {
    let video_id_for_db = video_id.clone();
    let res = tokio::task::spawn_blocking(move || {
        use crate::db::schema::bouts;
        let mut conn = db.get()?;
        let list = bouts::table
            .filter(bouts::video_id.eq(&video_id_for_db))
            .load::<Bout>(&mut conn)?;
        let sa: i32 = list.iter().map(|b| b.score_a).sum();
        let sb: i32 = list.iter().map(|b| b.score_b).sum();
        Ok::<_, anyhow::Error>((sa, sb))
    })
    .await;

    if let Ok(Ok((score_a, score_b))) = res {
        let _ = ws_hub.send(WsEvent::UpdateVideoScore {
            video_id,
            total_score_a: score_a,
            total_score_b: score_b,
        });
    }
}

pub async fn post_bout(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<CreateBoutRequest>,
) -> Result<(StatusCode, Json<BoutResponse>), AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

    let (bout, notifications) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, videos, users};
        use diesel::dsl::max;

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let video_rec = videos::table
            .filter(videos::id.eq(&body.video_id))
            .first::<crate::db::models::Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let video_rec = match video_rec {
            Some(v) => v,
            None => return Err(AppError::NotFound),
        };

        if video_rec.is_analyzing {
            return Err(AppError::BadRequest("Видео в данный момент обрабатывается ИИ. Создание сходов недоступно до завершения анализа.".to_string()));
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
                is_ai: false,
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let bout = bouts::table
            .filter(bouts::video_id.eq(&body.video_id))
            .order(bouts::id.desc())
            .first::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Insert history record
        use crate::db::models::NewBoutHistory;
        use crate::db::schema::bout_history;

        let details = format!(
            "Время: {} — {}",
            format_ms(bout.time_start_ms),
            format_ms(bout.time_end_ms)
        );

        diesel::insert_into(bout_history::table)
            .values(&NewBoutHistory {
                bout_id: bout.id,
                user_id: user_id.clone(),
                action: "create".to_string(),
                details: Some(details),
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Clear AI label — a human is now editing this video's bouts
        diesel::update(videos::table.filter(videos::id.eq(&bout.video_id)))
            .set(videos::is_ai_labeled.eq(false))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut notifications = Vec::new();

        let video = videos::table
            .filter(videos::id.eq(&bout.video_id))
            .first::<crate::db::models::Video>(&mut conn)
            .optional()
            .unwrap_or(None);

        if let Some(video) = video {
            let get_display_name = |v: &crate::db::models::Video, c: &mut diesel::SqliteConnection| -> String {
                use crate::db::schema::users;
                let date_str = v.date.format("%d.%m.%Y").to_string();
                
                let name_a = v.fighter_a_id.as_ref().and_then(|id_a| {
                    users::table.filter(users::id.eq(id_a))
                        .first::<crate::db::models::User>(c)
                        .ok()
                        .map(|u| u.display_name)
                });
                
                let name_b = v.fighter_b_id.as_ref().and_then(|id_b| {
                    users::table.filter(users::id.eq(id_b))
                        .first::<crate::db::models::User>(c)
                        .ok()
                        .map(|u| u.display_name)
                });
                
                match (name_a, name_b) {
                    (Some(a), Some(b)) => format!("{} vs {} ({})", a, b, date_str),
                    (Some(a), None) => format!("{} ({})", a, date_str),
                    (None, Some(b)) => format!("{} ({})", b, date_str),
                    (None, None) => format!("Без названия ({})", date_str),
                }
            };

            let video_title = get_display_name(&video, &mut conn);

            let mut participants = Vec::new();
            if let Some(ref fid) = video.fighter_a_id {
                participants.push(fid.clone());
            }
            if let Some(ref fid) = video.fighter_b_id {
                participants.push(fid.clone());
            }

            for part_id in participants {
                if part_id != user_id {
                    if let Ok(part_user) = users::table.filter(users::id.eq(&part_id)).first::<User>(&mut conn) {
                        if let Some(ref vk_id_str) = part_user.vk_id {
                            if !vk_id_str.trim().is_empty() {
                                notifications.push((part_id.clone(), vk_id_str.clone(), video_title.clone()));
                            }
                        }
                    }
                }
            }
        }

        Ok::<_, AppError>((bout, notifications))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(WsEvent::UpdateBout(to_ws_bout(&bout)));
    tokio::spawn(broadcast_video_score(state.db.clone(), state.ws_hub.clone(), bout.video_id.clone()));

    // Spawn VK notification tasks if not throttled
    let vk_notifier = state.vk_notifier.clone();
    let frontend_origin = state.frontend_url.clone();
    let video_id = bout.video_id.clone();
    for (part_id, vk_id, video_title) in notifications {
        if vk_notifier.check_outcome_throttle(&part_id, &video_id) {
            let notifier = vk_notifier.clone();
            let msg = format!(
                "⚔️ В вашем бою {} добавлены новые сходы.\n\nСсылка: {}/#/player/{}",
                video_title,
                frontend_origin,
                video_id
            );
            tokio::spawn(async move {
                notifier.send_notification(&vk_id, &msg).await;
            });
        }
    }

    Ok((StatusCode::CREATED, Json(to_response(&bout))))
}

pub async fn patch_bout(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i32>,
    Json(body): Json<PatchBoutRequest>,
) -> Result<Json<BoutResponse>, AppError> {
    let db = state.db.clone();
    let user_id = user.id.clone();

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
        let hit_zone_a = body.hit_zone_a.unwrap_or_else(|| cur.hit_zone_a.clone());
        let hit_zone_b = body.hit_zone_b.unwrap_or_else(|| cur.hit_zone_b.clone());
        let result_a = body.result_a.unwrap_or_else(|| cur.result_a.clone());
        let result_b = body.result_b.unwrap_or_else(|| cur.result_b.clone());

        // Compare and build changes list
        let mut changes = Vec::new();
        if time_start_ms != cur.time_start_ms {
            changes.push(format!("Время начала: {} → {}", format_ms(cur.time_start_ms), format_ms(time_start_ms)));
        }
        if time_end_ms != cur.time_end_ms {
            changes.push(format!("Время конца: {} → {}", format_ms(cur.time_end_ms), format_ms(time_end_ms)));
        }
        if score_a != cur.score_a {
            changes.push(format!("Очки А: {} → {}", cur.score_a, score_a));
        }
        if score_b != cur.score_b {
            changes.push(format!("Очки B: {} → {}", cur.score_b, score_b));
        }

        if technique_a_id != cur.technique_a_id || technique_b_id != cur.technique_b_id {
            use crate::db::schema::techniques;
            let mut tech_names = std::collections::HashMap::new();
            if let Ok(list) = techniques::table.load::<crate::db::models::Technique>(&mut conn) {
                for t in list {
                    tech_names.insert(t.id, t.name);
                }
            }

            let get_tech_name = |id_opt: Option<i32>| -> String {
                id_opt.and_then(|id| tech_names.get(&id).cloned()).unwrap_or_else(|| "—".to_string())
            };

            if technique_a_id != cur.technique_a_id {
                changes.push(format!("Прием А: {} → {}", get_tech_name(cur.technique_a_id), get_tech_name(technique_a_id)));
            }
            if technique_b_id != cur.technique_b_id {
                changes.push(format!("Прием B: {} → {}", get_tech_name(cur.technique_b_id), get_tech_name(technique_b_id)));
            }
        }

        let get_str_val = |val_opt: &Option<String>| -> String {
            val_opt.as_deref().unwrap_or("—").to_string()
        };

        if hit_zone_a != cur.hit_zone_a {
            changes.push(format!("Зона поражения А: {} → {}", get_str_val(&cur.hit_zone_a), get_str_val(&hit_zone_a)));
        }
        if hit_zone_b != cur.hit_zone_b {
            changes.push(format!("Зона поражения B: {} → {}", get_str_val(&cur.hit_zone_b), get_str_val(&hit_zone_b)));
        }

        let translate_result = |res: &Option<String>| -> String {
            match res.as_deref() {
                Some("hit") => "Попал",
                Some("miss") => "Промах",
                Some("blocked") => "Заблок.",
                Some("late") => "Опоздал",
                Some("no_strike") => "Не бил",
                Some("disqualification") => "Неквал.",
                Some("afterblow") => "Афтерблоу",
                Some(other) => other,
                None => "—",
            }.to_string()
        };

        if result_a != cur.result_a {
            changes.push(format!("Результат А: {} → {}", translate_result(&cur.result_a), translate_result(&result_a)));
        }
        if result_b != cur.result_b {
            changes.push(format!("Результат B: {} → {}", translate_result(&cur.result_b), translate_result(&result_b)));
        }

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
                bouts::is_ai.eq(false),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Write to history if there are changes
        if !changes.is_empty() {
            use crate::db::models::NewBoutHistory;
            use crate::db::schema::bout_history;

            diesel::insert_into(bout_history::table)
                .values(&NewBoutHistory {
                    bout_id: id,
                    user_id: user_id.clone(),
                    action: "update".to_string(),
                    details: Some(changes.join("; ")),
                })
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        // Clear AI label — human edited this bout
        {
            use crate::db::schema::videos;
            diesel::update(videos::table.filter(videos::id.eq(&cur.video_id)))
                .set(videos::is_ai_labeled.eq(false))
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        bouts::table
            .filter(bouts::id.eq(id))
            .first::<Bout>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let _ = state.ws_hub.send(WsEvent::UpdateBout(to_ws_bout(&bout)));
    tokio::spawn(broadcast_video_score(state.db.clone(), state.ws_hub.clone(), bout.video_id.clone()));

    Ok(Json(to_response(&bout)))
}

pub async fn delete_bout(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.db.clone();

    let bout = tokio::task::spawn_blocking(move || {
        use crate::db::schema::bouts;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        // Fetch bout before deletion to know its video_id and details
        let bout = bouts::table
            .filter(bouts::id.eq(id))
            .first::<Bout>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let deleted = diesel::delete(bouts::table.filter(bouts::id.eq(id)))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if deleted == 0 {
            return Err(AppError::NotFound);
        }

        // Clear AI label — human deleted a bout
        {
            use crate::db::schema::videos;
            diesel::update(videos::table.filter(videos::id.eq(&bout.video_id)))
                .set(videos::is_ai_labeled.eq(false))
                .execute(&mut conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        Ok::<Bout, AppError>(bout)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    // Send WebSocket notification that bout is deleted
    let mut ws_bout = to_ws_bout(&bout);
    ws_bout.deleted = Some(true);
    let _ = state.ws_hub.send(WsEvent::UpdateBout(ws_bout));

    // Broadcast updated video score
    tokio::spawn(broadcast_video_score(state.db.clone(), state.ws_hub.clone(), bout.video_id.clone()));

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Serialize)]
pub struct BoutHistoryUserResponse {
    pub id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct BoutHistoryResponse {
    pub id: i32,
    pub bout_id: i32,
    pub user: BoutHistoryUserResponse,
    pub action: String,
    pub details: Option<String>,
    pub created_at: String,
}

pub async fn get_bout_history(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<i32>,
) -> Result<Json<Vec<BoutHistoryResponse>>, AppError> {
    let db = state.db.clone();

    let history = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bout_history, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        // Query history joined with users
        let list = bout_history::table
            .filter(bout_history::bout_id.eq(id))
            .inner_join(users::table)
            .order(bout_history::created_at.desc())
            .load::<(crate::db::models::BoutHistory, crate::db::models::User)>(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let res = list
            .into_iter()
            .map(|(bh, u)| BoutHistoryResponse {
                id: bh.id,
                bout_id: bh.bout_id,
                user: BoutHistoryUserResponse {
                    id: u.id.clone(),
                    display_name: u.display_name.clone(),
                    avatar_url: format!("/api/users/{}/avatar", u.id),
                    color: u.color.clone(),
                },
                action: bh.action,
                details: bh.details,
                created_at: bh.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            })
            .collect::<Vec<_>>();

        Ok(res)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(history))
}


pub async fn download_bout_impl(
    state: &AppState,
    id: i32,
) -> Result<axum::response::Response, AppError> {
    let db = state.db.clone();

    let (bout, video, fighter_a, fighter_b) = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, videos, users};

        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;

        let bout = bouts::table
            .filter(bouts::id.eq(id))
            .first::<Bout>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let video = videos::table
            .filter(videos::id.eq(&bout.video_id))
            .first::<Video>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;

        let fighter_a = if let Some(ref fid) = video.fighter_a_id {
            users::table
                .filter(users::id.eq(fid))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            None
        };

        let fighter_b = if let Some(ref fid) = video.fighter_b_id {
            users::table
                .filter(users::id.eq(fid))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            None
        };

        Ok::<_, AppError>((bout, video, fighter_a, fighter_b))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let filename = {
        let name_a = fighter_a.map(|u| u.display_name).unwrap_or_else(|| "FighterA".to_string());
        let name_b = fighter_b.map(|u| u.display_name).unwrap_or_else(|| "FighterB".to_string());
        let clean_a = transliterate(&name_a);
        let clean_b = transliterate(&name_b);
        let clean_a = if clean_a.is_empty() { "FighterA".to_string() } else { clean_a };
        let clean_b = if clean_b.is_empty() { "FighterB".to_string() } else { clean_b };

        let date_str = video.date.format("%Y-%m-%d").to_string();
        let video_number = std::path::Path::new(&video.seafile_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");
        let clean_video_number = transliterate(video_number);
        let clean_video_number = if clean_video_number.is_empty() { "video".to_string() } else { clean_video_number };

        format!("{}_vs_{}_{}_{}_shod_{}.mp4", clean_a, clean_b, date_str, clean_video_number, bout.order_index)
    };

    let local_stream_url = format!("http://127.0.0.1:{}/api/videos/{}/stream", state.server_port, video.id);

    let temp_file = std::env::temp_dir().join(format!("bout_cut_{}.mp4", uuid::Uuid::new_v4()));
    let start_sec = (bout.time_start_ms as f64) / 1000.0;
    let duration_sec = ((bout.time_end_ms - bout.time_start_ms) as f64) / 1000.0;

    tracing::info!("Slicing bout {id} from {start_sec}s for {duration_sec}s into {:?}", temp_file);

    const FAKE_USER_AGENT: &str = "Mozilla/5.0";
    let ffmpeg_res = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss").arg(start_sec.to_string())
        .arg("-user_agent").arg(FAKE_USER_AGENT)
        .arg("-i").arg(&local_stream_url)
        .arg("-t").arg(duration_sec.to_string())
        .arg("-c").arg("copy")
        .arg(&temp_file)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("ffmpeg spawn: {e}")))?;

    if !ffmpeg_res.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_res.stderr);
        let code = ffmpeg_res.status.code().unwrap_or(-1);
        tracing::error!("ffmpeg slice bout FAILED (exit={code}):\n{stderr:.500}");
        return Err(AppError::Internal(format!("ffmpeg exit={code}")));
    }

    let bytes = tokio::fs::read(&temp_file)
        .await
        .map_err(|e| AppError::Internal(format!("read temp file: {e}")))?;
    
    let _ = tokio::fs::remove_file(&temp_file).await;

    let headers = [
        (header::CONTENT_TYPE, "video/mp4".to_string()),
        (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    ];

    Ok((headers, bytes).into_response())
}

pub async fn download_bout(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    download_bout_impl(&state, id).await
}

#[derive(Deserialize)]
pub struct SharedBoutQuery {
    pub token: String,
}

pub async fn download_shared_bout(
    State(state): State<AppState>,
    Path(bout_id): Path<i32>,
    Query(query): Query<SharedBoutQuery>,
) -> Result<axum::response::Response, AppError> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    let claims: crate::api::auth::ShareClaims = decode::<crate::api::auth::ShareClaims>(
        &query.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid share token: {}", e)))?;

    // We must query the bout to verify its video_id matches claims.video_id
    let db = state.db.clone();
    let bout_id_clone = bout_id;
    let bout_video_id = tokio::task::spawn_blocking(move || {
        use crate::db::schema::bouts;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let bout = bouts::table
            .filter(bouts::id.eq(bout_id_clone))
            .first::<Bout>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound)?;
        Ok::<String, AppError>(bout.video_id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if claims.video_id != bout_video_id {
        return Err(AppError::Unauthorized("Invalid share token for this bout's video".to_string()));
    }

    if let Some(tok_bout_id) = claims.bout_id {
        if tok_bout_id != bout_id {
            return Err(AppError::Unauthorized("Invalid share token for this fragment".to_string()));
        }
    }

    download_bout_impl(&state, bout_id).await
}


pub fn transliterate(s: &str) -> String {
    let mut res = String::new();
    for c in s.chars() {
        let mapped = match c {
            'а' => "a", 'б' => "b", 'в' => "v", 'г' => "g", 'д' => "d", 'е' => "e", 'ё' => "yo",
            'ж' => "zh", 'з' => "z", 'и' => "i", 'й' => "y", 'к' => "k", 'л' => "l", 'м' => "m",
            'н' => "n", 'о' => "o", 'п' => "p", 'р' => "r", 'с' => "s", 'т' => "t", 'у' => "u",
            'ф' => "f", 'х' => "kh", 'ц' => "ts", 'ч' => "ch", 'ш' => "sh", 'щ' => "shch",
            'ъ' => "", 'ы' => "y", 'ь' => "", 'э' => "e", 'ю' => "yu", 'я' => "ya",
            'А' => "A", 'Б' => "B", 'В' => "V", 'Г' => "G", 'Д' => "D", 'Е' => "E", 'Ё' => "Yo",
            'Ж' => "Zh", 'З' => "Z", 'И' => "I", 'Й' => "Y", 'К' => "K", 'Л' => "L", 'М' => "M",
            'Н' => "N", 'О' => "O", 'П' => "P", 'Р' => "R", 'С' => "S", 'Т' => "T", 'У' => "U",
            'Ф' => "F", 'Х' => "Kh", 'Ц' => "Ts", 'Ч' => "Ch", 'Ш' => "Sh", 'Щ' => "Shch",
            'Ъ' => "", 'Ы' => "Y", 'Ь' => "", 'Э' => "E", 'Ю' => "Yu", 'Я' => "Ya",
            _ if c.is_ascii_alphanumeric() => {
                res.push(c);
                continue;
            }
            ' ' | '_' | '-' => "_",
            _ => "",
        };
        res.push_str(mapped);
    }
    while res.contains("__") {
        res = res.replace("__", "_");
    }
    res.trim_matches('_').to_string()
}

pub fn format_ms(ms: i32) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}


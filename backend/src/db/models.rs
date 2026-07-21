use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::{bouts, comment_reactions, comments, techniques, users, videos, bout_history};

// ── users ─────────────────────────────────────────────────────────────────────

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub avatar_path: Option<String>,
    pub color: Option<String>,
    pub created_at: NaiveDateTime,
    pub vk_id: Option<String>,
    pub role: String,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub avatar_path: Option<String>,
    pub color: Option<String>,
    pub vk_id: Option<String>,
    pub role: String,
}

// ── videos ────────────────────────────────────────────────────────────────────

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = videos)]
pub struct Video {
    pub id: String,
    pub seafile_path: String,
    pub fighter_a_id: Option<String>,
    pub fighter_b_id: Option<String>,
    pub date: NaiveDate,
    pub duration_ms: Option<i32>,
    pub preview_count: i32,
    pub fps: Option<f32>,
    pub created_at: NaiveDateTime,
    pub is_ai_labeled: bool,
    pub is_analyzing: bool,
    pub is_queued: bool,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = videos)]
pub struct NewVideo {
    pub id: String,
    pub seafile_path: String,
    pub fighter_a_id: Option<String>,
    pub fighter_b_id: Option<String>,
    pub date: NaiveDate,
    pub duration_ms: Option<i32>,
    pub preview_count: i32,
    pub fps: Option<f32>,
}

// ── techniques ────────────────────────────────────────────────────────────────

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = techniques)]
pub struct Technique {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = techniques)]
pub struct NewTechnique {
    pub name: String,
    pub description: Option<String>,
}

// ── bouts ─────────────────────────────────────────────────────────────────────

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = bouts)]
pub struct Bout {
    pub id: i32,
    pub video_id: String,
    pub order_index: i32,
    pub time_start_ms: i32,
    pub time_end_ms: i32,
    pub score_a: i32,
    pub score_b: i32,
    pub technique_a_id: Option<i32>,
    pub technique_b_id: Option<i32>,
    pub hit_zone_a: Option<String>,
    pub hit_zone_b: Option<String>,
    pub result_a: Option<String>,
    pub result_b: Option<String>,
    pub is_ai: bool,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = bouts)]
pub struct NewBout {
    pub video_id: String,
    pub order_index: i32,
    pub time_start_ms: i32,
    pub time_end_ms: i32,
    pub score_a: i32,
    pub score_b: i32,
    pub technique_a_id: Option<i32>,
    pub technique_b_id: Option<i32>,
    pub hit_zone_a: Option<String>,
    pub hit_zone_b: Option<String>,
    pub result_a: Option<String>,
    pub result_b: Option<String>,
    pub is_ai: bool,
}

// ── comments ──────────────────────────────────────────────────────────────────

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: i32,
    pub video_id: String,
    pub author_id: String,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub edited_at: Option<NaiveDateTime>,
    pub guest_nickname: Option<String>,
    pub drawing: Option<String>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = comments)]
pub struct NewComment {
    pub video_id: String,
    pub author_id: String,
    pub timestamp_ms: i32,
    pub text: String,
    pub reply_to_id: Option<i32>,
    pub guest_nickname: Option<String>,
    pub drawing: Option<String>,
}

// ── comment_reactions ─────────────────────────────────────────────────────────

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = comment_reactions)]
pub struct CommentReaction {
    pub comment_id: i32,
    pub user_id: String,
    pub kind: String,
}

// ── bout_history ──────────────────────────────────────────────────────────────

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug, Associations)]
#[diesel(belongs_to(Bout, foreign_key = bout_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(table_name = bout_history)]
pub struct BoutHistory {
    pub id: i32,
    pub bout_id: i32,
    pub user_id: String,
    pub action: String,
    pub details: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = bout_history)]
pub struct NewBoutHistory {
    pub bout_id: i32,
    pub user_id: String,
    pub action: String,
    pub details: Option<String>,
}

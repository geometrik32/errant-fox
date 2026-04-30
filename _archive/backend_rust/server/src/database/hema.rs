use diesel::prelude::*;
use super::schema::*;
use super::models::*;
use super::{PooledConnection, DBResult};

impl HemaBout {
    pub fn get_by_media_file(conn: &mut PooledConnection, vid: &str) -> DBResult<Vec<HemaBout>> {
        hema_bouts::table
            .filter(hema_bouts::video_hash.eq(vid))
            .order(hema_bouts::start_time.asc())
            .load(conn)
            .map_err(|e| anyhow::anyhow!("Failed to fetch HEMA bouts for media {}: {:?}", vid, e).into())
    }
}

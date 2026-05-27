pub mod models;
pub mod schema;

use diesel::r2d2::{self, ConnectionManager, CustomizeConnection};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Apply PRAGMAs to every connection the pool creates — not just the one
/// used for migrations.  Without this, r2d2 connections use the default
/// rollback journal and concurrent writes fail with "database is locked".
#[derive(Debug)]
struct PragmaCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for PragmaCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        let map_err = |e| {
            diesel::r2d2::Error::ConnectionError(
                diesel::ConnectionError::CouldntSetupConfiguration(e),
            )
        };
        let journal_mode = sqlite_journal_mode();
        diesel::sql_query(format!("PRAGMA journal_mode={journal_mode}"))
            .execute(conn)
            .map_err(map_err)?;
        diesel::sql_query("PRAGMA busy_timeout=5000")
            .execute(conn)
            .map_err(map_err)?;
        diesel::sql_query("PRAGMA foreign_keys=ON")
            .execute(conn)
            .map_err(map_err)?;
        Ok(())
    }
}

fn sqlite_journal_mode() -> String {
    match env::var("SQLITE_JOURNAL_MODE") {
        Ok(mode) => {
            let normalized = mode.trim().to_ascii_uppercase();
            match normalized.as_str() {
                "DELETE" | "TRUNCATE" | "PERSIST" | "MEMORY" | "WAL" | "OFF" => normalized,
                _ => "WAL".to_string(),
            }
        }
        Err(_) => "WAL".to_string(),
    }
}

pub fn init_pool(database_url: &str) -> DbPool {
    // Run migrations on a direct connection before creating the pool.
    let mut conn = SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("failed to connect to {database_url}"));
    diesel::sql_query(format!("PRAGMA journal_mode={}", sqlite_journal_mode()))
        .execute(&mut conn)
        .expect("failed to set sqlite journal mode");
    diesel::sql_query("PRAGMA busy_timeout=5000")
        .execute(&mut conn)
        .expect("failed to set sqlite busy timeout");
    diesel::sql_query("PRAGMA foreign_keys=ON")
        .execute(&mut conn)
        .expect("failed to enable sqlite foreign keys");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run database migrations");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(4)
        .connection_customizer(Box::new(PragmaCustomizer))
        .build(manager)
        .expect("failed to create database connection pool")
}

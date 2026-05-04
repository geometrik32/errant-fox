pub mod models;
pub mod schema;

use diesel::r2d2::{self, ConnectionManager};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn init_pool(database_url: &str) -> DbPool {
    // Run migrations on a direct connection before creating the pool
    let mut conn = SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("failed to connect to {database_url}"));

    // Enable WAL mode for better concurrent read/write performance.
    // Without WAL, writers take an exclusive file lock and concurrent writes
    // fail with "database is locked".
    diesel::sql_query("PRAGMA journal_mode=WAL")
        .execute(&mut conn)
        .expect("failed to enable WAL mode");

    // Wait up to 5 seconds instead of immediately returning SQLITE_BUSY.
    diesel::sql_query("PRAGMA busy_timeout=5000")
        .execute(&mut conn)
        .expect("failed to set busy_timeout");

    // Enable foreign key enforcement.
    diesel::sql_query("PRAGMA foreign_keys=ON")
        .execute(&mut conn)
        .expect("failed to enable foreign keys");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run database migrations");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("failed to create database connection pool")
}

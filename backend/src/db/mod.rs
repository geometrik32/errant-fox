pub mod models;
pub mod schema;

use diesel::r2d2::{self, ConnectionManager};
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn init_pool(database_url: &str) -> DbPool {
    // Run migrations on a direct connection before creating the pool
    let mut conn = SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("failed to connect to {database_url}"));
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run database migrations");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create database connection pool")
}

//! Connection pool construction and migration running.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Settings;

/// Migrations embedded in the binary at compile time, so a release image needs
/// no migration files on disk and cannot drift from the code it ships with.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Opens the connection pool. `pool_pre_ping`'s equivalent is sqlx's default
/// test-before-acquire behaviour, so a connection dropped by Postgres is
/// replaced rather than handed to a request.
pub async fn connect(settings: &Settings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(settings.database_max_connections)
        .connect(&settings.database_url)
        .await
}

/// Applies any migrations the database has not yet run.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

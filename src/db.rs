use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

/// # Errors
///
/// Returns an error if the connection string is invalid or the database cannot be opened.
pub async fn create_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    if should_create_parent_dirs(db_path)
        && let Some(parent) = Path::new(db_path).parent()
    {
        std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
    }

    let options = SqliteConnectOptions::from_str(db_path)?.create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

/// # Errors
///
/// Returns an error if any migration fails to apply.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

fn should_create_parent_dirs(db_path: &str) -> bool {
    if db_path == ":memory:" {
        return false;
    }

    if db_path.starts_with("sqlite:") || db_path.starts_with("file:") {
        return false;
    }

    Path::new(db_path)
        .parent()
        .is_some_and(|p| !p.as_os_str().is_empty())
}

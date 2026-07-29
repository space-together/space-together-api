use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Clone)]
pub struct PgManager {
    pub pool: PgPool,
}

impl PgManager {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        ensure_database_exists(database_url).await?;

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

async fn ensure_database_exists(database_url: &str) -> anyhow::Result<()> {
    let db_name = extract_db_name(database_url).ok_or_else(|| {
        anyhow::anyhow!("Cannot parse database name from DB_URL")
    })?;

    let admin_url = replace_db_name(database_url, "postgres");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&admin_url)
        .await?;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)"
    )
    .bind(&db_name)
    .fetch_one(&pool)
    .await?;

    if !exists {
        let create_sql = format!(r#"CREATE DATABASE "{}""#, db_name);
        sqlx::query(&create_sql).execute(&pool).await?;
        log::info!("Created database '{}'", db_name);
    }

    pool.close().await;
    Ok(())
}

fn extract_db_name(url: &str) -> Option<String> {
    url.rsplit('/').next().map(|s| {
        // strip query params
        s.split('?').next().unwrap_or(s).to_string()
    })
}

fn replace_db_name(url: &str, new_db: &str) -> String {
    if let Some(idx) = url.rfind('/') {
        let after_slash = &url[idx + 1..];
        let query_part = after_slash.find('?').map(|q| &after_slash[q..]).unwrap_or("");
        format!("{}/{}{}", &url[..idx], new_db, query_part)
    } else {
        url.to_string()
    }
}

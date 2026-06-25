// Database seeder: populates every table with default demo data.
//
// Usage:
//   cargo run --bin seed            -> run migrations, then execute scripts/seed.sql
//   cargo run --bin seed -- schema  -> print live schema (required columns + FKs) per table
//
// The seed SQL is idempotent (ON CONFLICT DO NOTHING), so it can be run repeatedly.
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::Duration;

fn database_url() -> anyhow::Result<String> {
    let raw = std::env::var("DB_URL")?;
    // The app stores a non-standard scheme (e.g. "localhost://"); normalize it so
    // sqlx accepts the URL.
    let normalized = if raw.starts_with("postgres://") || raw.starts_with("postgresql://") {
        raw
    } else if let Some(rest) = raw.split_once("://") {
        format!("postgres://{}", rest.1)
    } else {
        raw
    };
    Ok(normalized)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let url = database_url()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    if std::env::args().any(|a| a == "schema") {
        dump_schema(&pool).await?;
        return Ok(());
    }

    if std::env::args().any(|a| a == "verify") {
        verify_counts(&pool).await?;
        return Ok(());
    }

    let sql = include_str!("../../scripts/seed.sql");
    // Execute the whole script in one batch so statement ordering is preserved.
    use sqlx::Executor;
    let mut conn = pool.acquire().await?;
    conn.execute(sql).await?;
    println!("Seed complete: all tables populated with default data.");
    Ok(())
}

async fn verify_counts(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let tables = sqlx::query(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema='public' AND table_type='BASE TABLE' \
         AND table_name <> '_sqlx_migrations' ORDER BY table_name",
    )
    .fetch_all(pool)
    .await?;

    let mut empty = 0;
    let total = tables.len();
    for t in &tables {
        let table: String = t.get("table_name");
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM \"{table}\""))
            .fetch_one(pool)
            .await?;
        if count == 0 {
            empty += 1;
            println!("EMPTY  {table}");
        }
    }
    println!("\n{} tables total, {} empty.", total, empty);
    if empty == 0 {
        println!("OK: every table has data.");
    }
    Ok(())
}

async fn dump_schema(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let tables = sqlx::query(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema='public' AND table_type='BASE TABLE' \
         AND table_name <> '_sqlx_migrations' ORDER BY table_name",
    )
    .fetch_all(pool)
    .await?;

    for t in &tables {
        let table: String = t.get("table_name");
        println!("\n== {table} ==");

        let cols = sqlx::query(
            "SELECT column_name, data_type, is_nullable, column_default \
             FROM information_schema.columns \
             WHERE table_schema='public' AND table_name=$1 ORDER BY ordinal_position",
        )
        .bind(&table)
        .fetch_all(pool)
        .await?;

        for c in &cols {
            let name: String = c.get("column_name");
            let dtype: String = c.get("data_type");
            let nullable: String = c.get("is_nullable");
            let default: Option<String> = c.get("column_default");
            let required = nullable == "NO" && default.is_none();
            println!(
                "  {}{} {} {}",
                if required { "* " } else { "  " },
                name,
                dtype,
                if default.is_some() { "[has-default]" } else { "" }
            );
        }

        let fks = sqlx::query(
            "SELECT kcu.column_name, ccu.table_name AS ref_table \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu ON tc.constraint_name=kcu.constraint_name \
             JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name=ccu.constraint_name \
             WHERE tc.constraint_type='FOREIGN KEY' AND tc.table_name=$1",
        )
        .bind(&table)
        .fetch_all(pool)
        .await?;
        for f in &fks {
            let col: String = f.get("column_name");
            let ref_table: String = f.get("ref_table");
            println!("  FK {col} -> {ref_table}");
        }
    }
    println!("\nLegend: '*' = required (NOT NULL, no default)");
    Ok(())
}

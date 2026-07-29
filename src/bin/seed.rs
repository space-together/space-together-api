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

    // Set a shared default password on every seeded user. The hash is generated
    // with the same Argon2id params the app uses (m=4096,t=2,p=1) so login and
    // the rehash check both accept it.
    let password = std::env::var("SEED_PASSWORD").unwrap_or_else(|_| "Demo@1234".to_string());
    let hash = hash_password(&password);
    let updated = sqlx::query("UPDATE users SET password_hash = $1 WHERE password_hash IS NULL")
        .bind(&hash)
        .execute(&pool)
        .await?
        .rows_affected();

    println!("Seed complete: all tables populated with default data.");
    println!("Default password set on {updated} user(s): '{password}'");
    Ok(())
}

// Mirrors src/utils/hash.rs (this bin cannot import the app crate, which is bin-only).
fn hash_password(password: &str) -> String {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::{Algorithm, Argon2, Params, Version};
    let params = Params::new(4_096, 2, 1, None).expect("valid Argon2 params");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hash password")
        .to_string()
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

use sqlx::{PgPool, Row};

use crate::{
    domain::database_status::{CollectionStats, DbStats},
    utils::bytes::format_bytes,
};

pub async fn get_postgres_stats(pool: &PgPool, school_id: Option<&str>) -> Result<DbStats, String> {
    let tables = sqlx::query(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_type = 'BASE TABLE'
        ORDER BY table_name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|err| err.to_string())?;

    let mut total_documents = 0_u64;
    let mut total_size_bytes = 0_usize;
    let mut collections = Vec::new();

    for table in tables {
        let table_name: String = table.try_get("table_name").map_err(|err| err.to_string())?;
        let qualified_name = format!("public.{}", table_name);

        let has_school_id: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
              SELECT 1
              FROM information_schema.columns
              WHERE table_schema = 'public'
                AND table_name = $1
                AND column_name = 'school_id'
            )
            "#,
        )
        .bind(&table_name)
        .fetch_one(pool)
        .await
        .map_err(|err| err.to_string())?;

        let count_sql = if school_id.is_some() && has_school_id {
            format!(
                "SELECT count(*)::BIGINT FROM \"{}\" WHERE school_id = $1",
                table_name
            )
        } else {
            format!("SELECT count(*)::BIGINT FROM \"{}\"", table_name)
        };

        let document_count: i64 = if let Some(school_id) = school_id.filter(|_| has_school_id) {
            sqlx::query_scalar(&count_sql)
                .bind(school_id)
                .fetch_one(pool)
                .await
                .map_err(|err| err.to_string())?
        } else {
            sqlx::query_scalar(&count_sql)
                .fetch_one(pool)
                .await
                .map_err(|err| err.to_string())?
        };

        let size_bytes: i64 = sqlx::query_scalar(
            "SELECT COALESCE(pg_total_relation_size(to_regclass($1)), 0)::BIGINT",
        )
        .bind(&qualified_name)
        .fetch_one(pool)
        .await
        .map_err(|err| err.to_string())?;

        let document_count = document_count.max(0) as u64;
        let size_bytes = size_bytes.max(0) as usize;
        total_documents += document_count;
        total_size_bytes += size_bytes;
        collections.push(CollectionStats {
            name: table_name,
            document_count,
            size_bytes: format_bytes(size_bytes),
        });
    }

    Ok(DbStats {
        total_documents,
        total_size_bytes: format_bytes(total_size_bytes),
        total_collection: collections.len(),
        collections,
    })
}

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        comment::{Comment, CommentPartial, CommentWithRelations},
        common_details::{Paginated, UserRole},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    schema::common_schema::ActorRef,
    utils::object_id::ObjectId,
};

pub struct CommentService {
    pub pool: PgPool,
}

impl CommentService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        Ok(())
    }

    fn db_error(error: sqlx::Error) -> AppError {
        AppError {
            message: format!("PostgreSQL Error: {}", error),
        }
    }

    fn new_id() -> String {
        ObjectId::new().to_hex()
    }

    fn id_to_string(id: &IdType) -> Result<String, AppError> {
        Ok(IdType::to_object_id(id)?.to_hex())
    }

    fn parse_oid(raw: &str, field: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(raw).map_err(|e| AppError {
            message: format!("Invalid {} ObjectId-compatible ID: {}", field, e),
        })
    }

    fn parse_oid_opt(raw: Option<String>, field: &str) -> Result<Option<ObjectId>, AppError> {
        raw.map(|value| Self::parse_oid(&value, field)).transpose()
    }

    fn role_to_string(role: &UserRole) -> String {
        role.to_string()
    }

    fn role_from_string(raw: Option<String>) -> UserRole {
        match raw
            .unwrap_or_else(|| "STUDENT".to_string())
            .to_ascii_uppercase()
            .as_str()
        {
            "TEACHER" => UserRole::TEACHER,
            "ADMIN" => UserRole::ADMIN,
            "SCHOOLSTAFF" => UserRole::SCHOOLSTAFF,
            "PARENT" => UserRole::PARENT,
            _ => UserRole::STUDENT,
        }
    }

    fn select_sql() -> &'static str {
        r#"
        SELECT id, COALESCE(content, body, '') AS content,
               COALESCE(author_actor_id, author_user_id) AS author_actor_id,
               author_role,
               COALESCE(target_post_id, target_id) AS target_post_id,
               parent_comment_id,
               created_at, updated_at
        FROM comments
        WHERE deleted_at IS NULL
        "#
    }

    fn comment_from_row(row: &sqlx::postgres::PgRow) -> Result<Comment, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let author_id: String = row.try_get("author_actor_id").map_err(Self::db_error)?;
        let target_post_id: String = row.try_get("target_post_id").map_err(Self::db_error)?;

        Ok(Comment {
            id: Some(Self::parse_oid(&id, "id")?),
            content: row.try_get("content").map_err(Self::db_error)?,
            author: ActorRef {
                id: Self::parse_oid(&author_id, "author_id")?,
                role: Self::role_from_string(row.try_get("author_role").ok().flatten()),
            },
            target_post_id: Self::parse_oid(&target_post_id, "target_post_id")?,
            parent_comment_id: Self::parse_oid_opt(
                row.try_get("parent_comment_id").ok().flatten(),
                "parent_comment_id",
            )?,
            created_at: row
                .try_get::<Option<DateTime<Utc>>, _>("created_at")
                .ok()
                .flatten(),
            updated_at: row
                .try_get::<Option<DateTime<Utc>>, _>("updated_at")
                .ok()
                .flatten(),
        })
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RequestQuery>,
        school_id: Option<&'a str>,
    ) -> Result<(), AppError> {
        if let Some(school_id) = school_id {
            Self::parse_oid(school_id, "school_id")?;
            sql.push(" AND school_id = ").push_bind(school_id);
        }

        let Some(query) = query else {
            return Ok(());
        };

        if !query.by_ids.is_empty() {
            sql.push(" AND id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                Self::parse_oid(id, "id")?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }

        for (field, value) in query.field.iter().zip(query.value.iter()) {
            match field.as_str() {
                "_id" | "id" => {
                    Self::parse_oid(value, "id")?;
                    sql.push(" AND id = ").push_bind(value);
                }
                "target_post_id" | "target_id" => {
                    Self::parse_oid(value, "target_post_id")?;
                    sql.push(" AND COALESCE(target_post_id, target_id) = ")
                        .push_bind(value);
                }
                "parent_comment_id" => {
                    Self::parse_oid(value, "parent_comment_id")?;
                    sql.push(" AND parent_comment_id = ").push_bind(value);
                }
                "author.id" | "author_id" | "author_actor_id" => {
                    Self::parse_oid(value, "author_id")?;
                    sql.push(" AND COALESCE(author_actor_id, author_user_id) = ")
                        .push_bind(value);
                }
                "content" => {
                    sql.push(" AND lower(COALESCE(content, body, '')) LIKE ")
                        .push_bind(format!("%{}%", value.to_ascii_lowercase()));
                }
                "school_id" => {
                    Self::parse_oid(value, "school_id")?;
                    sql.push(" AND school_id = ").push_bind(value);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, filter: &'a str) {
        let search = format!("%{}%", filter.to_ascii_lowercase());
        sql.push(" AND (lower(COALESCE(content, body, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(id) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(target_post_id, target_id, '')) LIKE ")
            .push_bind(search)
            .push(")");
    }

    pub async fn create(
        &self,
        dto: Comment,
        school_id: Option<String>,
    ) -> Result<Comment, AppError> {
        self.ensure_indexes().await?;
        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);
        let now = Utc::now();
        let created_at = dto.created_at.unwrap_or(now);
        let updated_at = dto.updated_at.unwrap_or(created_at);

        sqlx::query(
            r#"
            INSERT INTO comments (
              id, school_id, target_type, target_id, target_post_id,
              parent_comment_id, author_user_id, author_actor_id, author_role,
              body, content, created_at, updated_at
            )
            VALUES ($1, $2, 'post', $3, $3, $4, $5, $6, $7, $8, $8, $9, $10)
            "#,
        )
        .bind(&id)
        .bind(school_id)
        .bind(dto.target_post_id.to_hex())
        .bind(dto.parent_comment_id.map(|id| id.to_hex()))
        .bind(Option::<String>::None)
        .bind(dto.author.id.to_hex())
        .bind(Self::role_to_string(&dto.author.role))
        .bind(dto.content)
        .bind(created_at)
        .bind(updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::String(id)), None, None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Comment, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());

        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_query_filters(&mut sql, query, school_id)?;
        sql.push(" ORDER BY updated_at DESC LIMIT 1");

        sql.build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .map(|row| Self::comment_from_row(&row))
            .transpose()?
            .ok_or(AppError {
                message: "Comment not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<Comment>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_sql = QueryBuilder::<Postgres>::new(
            "SELECT count(*)::BIGINT FROM comments WHERE deleted_at IS NULL",
        );
        Self::push_query_filters(&mut count_sql, query, school_id)?;
        if let Some(filter) = filter.as_deref() {
            Self::push_search(&mut count_sql, filter);
        }
        let total: i64 = count_sql
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut sql, query, school_id)?;
        if let Some(filter) = filter.as_deref() {
            Self::push_search(&mut sql, filter);
        }
        sql.push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);

        let rows = sql
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;
        let data = rows
            .into_iter()
            .map(|row| Self::comment_from_row(&row))
            .collect::<Result<Vec<_>, _>>()?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as f64) / (limit as f64)).ceil() as i64
        };

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &CommentPartial) -> Result<Comment, AppError> {
        let id = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE comments SET updated_at = now()");

        if let Some(content) = &update.content {
            sql.push(", content = ")
                .push_bind(content)
                .push(", body = ")
                .push_bind(content);
        }
        if let Some(author) = &update.author {
            sql.push(", author_actor_id = ")
                .push_bind(author.id.to_hex())
                .push(", author_user_id = ")
                .push_bind(Option::<String>::None)
                .push(", author_role = ")
                .push_bind(Self::role_to_string(&author.role));
        }
        if let Some(target_post_id) = &update.target_post_id {
            sql.push(", target_post_id = ")
                .push_bind(target_post_id.to_hex())
                .push(", target_id = ")
                .push_bind(target_post_id.to_hex());
        }
        if let Some(parent_comment_id) = &update.parent_comment_id {
            sql.push(", parent_comment_id = ")
                .push_bind(parent_comment_id.map(|id| id.to_hex()));
        }

        sql.push(" WHERE id = ")
            .push_bind(&id)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one(Some(&IdType::String(id)), None, None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Comment, AppError> {
        let comment = self.find_one(Some(id), None, None).await?;
        sqlx::query("UPDATE comments SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(comment)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<CommentWithRelations>, AppError> {
        let page = self.get_all(filter, limit, skip, query, school_id).await?;
        Ok(Paginated {
            data: page
                .data
                .into_iter()
                .map(|comment| CommentWithRelations {
                    comment,
                    author_user: None,
                })
                .collect(),
            total: page.total,
            total_pages: page.total_pages,
            current_page: page.current_page,
        })
    }

    pub async fn count_comments(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<u64, AppError> {
        Ok(self
            .get_all(filter, Some(1), Some(0), query, school_id)
            .await?
            .total as u64)
    }

    pub async fn delete_many_by_target(&self, target_id: &ObjectId) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE comments
            SET deleted_at = now(), updated_at = now()
            WHERE COALESCE(target_post_id, target_id) = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(target_id.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(())
    }
}

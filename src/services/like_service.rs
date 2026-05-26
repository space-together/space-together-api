use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::{Paginated, UserRole},
        like::{Like, LikePartial, LikeWithRelations},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    schema::common_schema::ActorRef,
    utils::object_id::ObjectId,
};

pub struct LikeService {
    pub pool: PgPool,
}

impl LikeService {
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
        SELECT id, COALESCE(actor_id, user_id) AS actor_id, actor_role,
               target_id, COALESCE(like_value, 'like') AS like_value,
               created_at
        FROM likes
        WHERE deleted_at IS NULL
        "#
    }

    fn like_from_row(row: &sqlx::postgres::PgRow) -> Result<Like, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let actor_id: String = row.try_get("actor_id").map_err(Self::db_error)?;
        let target_id: String = row.try_get("target_id").map_err(Self::db_error)?;
        let created_at = row
            .try_get::<Option<DateTime<Utc>>, _>("created_at")
            .ok()
            .flatten();

        Ok(Like {
            id: Some(Self::parse_oid(&id, "id")?),
            actor: ActorRef {
                id: Self::parse_oid(&actor_id, "actor_id")?,
                role: Self::role_from_string(row.try_get("actor_role").ok().flatten()),
            },
            target_id: Self::parse_oid(&target_id, "target_id")?,
            like: row.try_get("like_value").ok().flatten(),
            created_at,
            updated_at: created_at,
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
                "target_id" => {
                    Self::parse_oid(value, "target_id")?;
                    sql.push(" AND target_id = ").push_bind(value);
                }
                "actor.id" | "actor_id" | "user_id" => {
                    Self::parse_oid(value, "actor_id")?;
                    sql.push(" AND COALESCE(actor_id, user_id) = ")
                        .push_bind(value);
                }
                "like" => {
                    sql.push(" AND lower(COALESCE(like_value, 'like')) = ")
                        .push_bind(value.to_ascii_lowercase());
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
        sql.push(" AND (lower(id) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(target_id) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(actor_id, user_id, '')) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(COALESCE(like_value, 'like')) LIKE ")
            .push_bind(search)
            .push(")");
    }

    async fn find_by_target_actor(
        &self,
        target_id: &str,
        actor_id: &str,
        actor_role: &str,
    ) -> Result<Like, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        sql.push(" AND target_id = ")
            .push_bind(target_id)
            .push(" AND COALESCE(actor_id, user_id) = ")
            .push_bind(actor_id)
            .push(" AND actor_role = ")
            .push_bind(actor_role)
            .push(" ORDER BY created_at DESC LIMIT 1");

        sql.build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .map(|row| Self::like_from_row(&row))
            .transpose()?
            .ok_or(AppError {
                message: "Like not found".into(),
            })
    }

    pub async fn create(&self, dto: Like, school_id: Option<String>) -> Result<Like, AppError> {
        self.ensure_indexes().await?;
        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);
        let created_at = dto.created_at.unwrap_or_else(Utc::now);
        let target_id = dto.target_id.to_hex();
        let actor_id = dto.actor.id.to_hex();
        let actor_role = Self::role_to_string(&dto.actor.role);
        let like_value = dto.like.unwrap_or_else(|| "like".to_string());

        let result = sqlx::query(
            r#"
            INSERT INTO likes (
              id, school_id, target_type, target_id, user_id, actor_id, actor_role,
              like_value, created_at
            )
            VALUES ($1, $2, 'post', $3, $4, $5, $6, $7, $8)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&id)
        .bind(school_id)
        .bind(&target_id)
        .bind(Option::<String>::None)
        .bind(&actor_id)
        .bind(&actor_role)
        .bind(like_value)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        if result.rows_affected() == 0 {
            return self
                .find_by_target_actor(&target_id, &actor_id, &actor_role)
                .await;
        }

        self.find_one(Some(&IdType::String(id)), None, None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Like, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_query_filters(&mut sql, query, school_id)?;
        sql.push(" ORDER BY created_at DESC LIMIT 1");

        sql.build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .map(|row| Self::like_from_row(&row))
            .transpose()?
            .ok_or(AppError {
                message: "Like not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<Like>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_sql = QueryBuilder::<Postgres>::new(
            "SELECT count(*)::BIGINT FROM likes WHERE deleted_at IS NULL",
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
            .map(|row| Self::like_from_row(&row))
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

    pub async fn update(&self, id: &IdType, update: &LikePartial) -> Result<Like, AppError> {
        let id = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE likes SET id = id");

        if let Some(actor) = &update.actor {
            sql.push(", actor_id = ")
                .push_bind(actor.id.to_hex())
                .push(", user_id = ")
                .push_bind(Option::<String>::None)
                .push(", actor_role = ")
                .push_bind(Self::role_to_string(&actor.role));
        }
        if let Some(target_id) = &update.target_id {
            sql.push(", target_id = ").push_bind(target_id.to_hex());
        }
        if let Some(like) = &update.like {
            sql.push(", like_value = ").push_bind(like);
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

    pub async fn delete(&self, id: &IdType) -> Result<Like, AppError> {
        let like = self.find_one(Some(id), None, None).await?;
        sqlx::query("UPDATE likes SET deleted_at = now() WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(like)
    }

    pub async fn get_all_with_relations(
        &self,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<LikeWithRelations>, AppError> {
        let page = self.get_all(None, limit, skip, query, school_id).await?;
        Ok(Paginated {
            data: page
                .data
                .into_iter()
                .map(|like| LikeWithRelations {
                    like,
                    author_user: None,
                })
                .collect(),
            total: page.total,
            total_pages: page.total_pages,
            current_page: page.current_page,
        })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<LikeWithRelations, AppError> {
        let like = self.find_one(id, query, school_id).await?;
        Ok(LikeWithRelations {
            like,
            author_user: None,
        })
    }

    pub async fn count_likes(
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
            "UPDATE likes SET deleted_at = now() WHERE target_id = $1 AND deleted_at IS NULL",
        )
        .bind(target_id.to_hex())
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(())
    }
}

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        announcement::{Announcement, AnnouncementPartial, AnnouncementWithRelations},
        common_details::{Paginated, UserRole},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    schema::common_schema::ActorRef,
    utils::object_id::ObjectId,
};

pub struct AnnouncementService {
    pub pool: PgPool,
}

impl AnnouncementService {
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
        SELECT a.id, COALESCE(a.body, '') AS content,
               COALESCE(a.published_actor_id, a.author_user_id) AS published_actor_id,
               a.published_role,
               ARRAY(
                 SELECT ac.class_id
                 FROM announcement_classes ac
                 WHERE ac.announcement_id = a.id
                 ORDER BY ac.class_id
               ) AS classes_ids,
               ARRAY(
                 SELECT am.actor_id || ':' || am.actor_role
                 FROM announcement_mentions am
                 WHERE am.announcement_id = a.id
                 ORDER BY am.actor_id, am.actor_role
               ) AS mentions,
               a.created_at, a.updated_at
        FROM announcements a
        WHERE a.deleted_at IS NULL
        "#
    }

    fn announcement_from_row(row: &sqlx::postgres::PgRow) -> Result<Announcement, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let published_id: String = row.try_get("published_actor_id").map_err(Self::db_error)?;
        let class_ids: Vec<String> = row.try_get("classes_ids").unwrap_or_default();
        let mentions: Vec<String> = row.try_get("mentions").unwrap_or_default();

        let classes_ids = if class_ids.is_empty() {
            None
        } else {
            Some(
                class_ids
                    .iter()
                    .map(|id| Self::parse_oid(id, "class_id"))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        };

        let mention = if mentions.is_empty() {
            None
        } else {
            let mut actors = Vec::new();
            for value in mentions {
                if let Some((actor_id, role)) = value.split_once(':') {
                    actors.push(ActorRef {
                        id: Self::parse_oid(actor_id, "mention.id")?,
                        role: Self::role_from_string(Some(role.to_string())),
                    });
                }
            }
            Some(actors)
        };

        Ok(Announcement {
            id: Some(Self::parse_oid(&id, "id")?),
            content: row.try_get("content").map_err(Self::db_error)?,
            mention,
            published: ActorRef {
                id: Self::parse_oid(&published_id, "published.id")?,
                role: Self::role_from_string(row.try_get("published_role").ok().flatten()),
            },
            classes_ids,
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
            sql.push(" AND a.school_id = ").push_bind(school_id);
        }

        let Some(query) = query else {
            return Ok(());
        };

        if let Some(class_id) = query.class_id.as_deref() {
            Self::parse_oid(class_id, "class_id")?;
            sql.push(
                " AND EXISTS (SELECT 1 FROM announcement_classes ac WHERE ac.announcement_id = a.id AND ac.class_id = ",
            )
            .push_bind(class_id)
            .push(")");
        }

        if !query.by_ids.is_empty() {
            sql.push(" AND a.id IN (");
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
                    sql.push(" AND a.id = ").push_bind(value);
                }
                "school_id" => {
                    Self::parse_oid(value, "school_id")?;
                    sql.push(" AND a.school_id = ").push_bind(value);
                }
                "classes_ids" | "class_id" => {
                    Self::parse_oid(value, "class_id")?;
                    sql.push(
                        " AND EXISTS (SELECT 1 FROM announcement_classes ac WHERE ac.announcement_id = a.id AND ac.class_id = ",
                    )
                    .push_bind(value)
                    .push(")");
                }
                "published.id" | "published_id" | "published_actor_id" => {
                    Self::parse_oid(value, "published.id")?;
                    sql.push(" AND COALESCE(a.published_actor_id, a.author_user_id) = ")
                        .push_bind(value);
                }
                "published.role" | "published_role" => {
                    sql.push(" AND lower(a.published_role) = ")
                        .push_bind(value.to_ascii_lowercase());
                }
                "mention.id" | "mention_id" => {
                    Self::parse_oid(value, "mention.id")?;
                    sql.push(
                        " AND EXISTS (SELECT 1 FROM announcement_mentions am WHERE am.announcement_id = a.id AND am.actor_id = ",
                    )
                    .push_bind(value)
                    .push(")");
                }
                "mention.role" | "mention_role" => {
                    sql.push(
                        " AND EXISTS (SELECT 1 FROM announcement_mentions am WHERE am.announcement_id = a.id AND lower(am.actor_role) = ",
                    )
                    .push_bind(value.to_ascii_lowercase())
                    .push(")");
                }
                "content" => {
                    sql.push(" AND lower(a.body) LIKE ")
                        .push_bind(format!("%{}%", value.to_ascii_lowercase()));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, filter: &'a str) {
        let search = format!("%{}%", filter.to_ascii_lowercase());
        sql.push(" AND (lower(a.body) LIKE ")
            .push_bind(search.clone())
            .push(" OR lower(a.id) LIKE ")
            .push_bind(search.clone())
            .push(
                " OR EXISTS (SELECT 1 FROM announcement_classes ac WHERE ac.announcement_id = a.id AND lower(ac.class_id) LIKE ",
            )
            .push_bind(search)
            .push("))");
    }

    async fn replace_classes(
        &self,
        announcement_id: &str,
        class_ids: Option<&Vec<ObjectId>>,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM announcement_classes WHERE announcement_id = $1")
            .bind(announcement_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        if let Some(class_ids) = class_ids {
            for class_id in class_ids {
                sqlx::query(
                    r#"
                    INSERT INTO announcement_classes (announcement_id, class_id)
                    VALUES ($1, $2)
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(announcement_id)
                .bind(class_id.to_hex())
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
            }
        }

        Ok(())
    }

    async fn replace_mentions(
        &self,
        announcement_id: &str,
        mentions: Option<&Vec<ActorRef>>,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM announcement_mentions WHERE announcement_id = $1")
            .bind(announcement_id)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        if let Some(mentions) = mentions {
            for mention in mentions {
                sqlx::query(
                    r#"
                    INSERT INTO announcement_mentions (announcement_id, actor_id, actor_role)
                    VALUES ($1, $2, $3)
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(announcement_id)
                .bind(mention.id.to_hex())
                .bind(Self::role_to_string(&mention.role))
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
            }
        }

        Ok(())
    }

    pub async fn create(
        &self,
        dto: Announcement,
        school_id: Option<String>,
    ) -> Result<Announcement, AppError> {
        self.ensure_indexes().await?;
        let id = dto.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);
        let now = Utc::now();
        let created_at = dto.created_at.unwrap_or(now);
        let updated_at = dto.updated_at.unwrap_or(created_at);
        let class_id = dto
            .classes_ids
            .as_ref()
            .and_then(|ids| ids.first())
            .map(|id| id.to_hex());

        sqlx::query(
            r#"
            INSERT INTO announcements (
              id, school_id, class_id, author_user_id, published_actor_id,
              published_role, title, body, audience, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'Announcement', $7, 'school', $8, $9)
            "#,
        )
        .bind(&id)
        .bind(school_id)
        .bind(class_id)
        .bind(Option::<String>::None)
        .bind(dto.published.id.to_hex())
        .bind(Self::role_to_string(&dto.published.role))
        .bind(dto.content)
        .bind(created_at)
        .bind(updated_at)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.replace_classes(&id, dto.classes_ids.as_ref()).await?;
        self.replace_mentions(&id, dto.mention.as_ref()).await?;

        self.find_one(Some(&IdType::String(id)), None, None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Announcement, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND a.id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_query_filters(&mut sql, query, school_id)?;
        sql.push(" ORDER BY a.updated_at DESC LIMIT 1");

        sql.build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?
            .map(|row| Self::announcement_from_row(&row))
            .transpose()?
            .ok_or(AppError {
                message: "Announcement not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<Announcement>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut count_sql = QueryBuilder::<Postgres>::new(
            "SELECT count(*)::BIGINT FROM announcements a WHERE a.deleted_at IS NULL",
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
        sql.push(" ORDER BY a.created_at DESC LIMIT ")
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
            .map(|row| Self::announcement_from_row(&row))
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

    pub async fn update(
        &self,
        id: &IdType,
        update: &AnnouncementPartial,
    ) -> Result<Announcement, AppError> {
        let id = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE announcements SET updated_at = now()");

        if let Some(content) = &update.content {
            sql.push(", body = ").push_bind(content);
        }
        if let Some(published) = &update.published {
            sql.push(", published_actor_id = ")
                .push_bind(published.id.to_hex())
                .push(", author_user_id = ")
                .push_bind(Option::<String>::None)
                .push(", published_role = ")
                .push_bind(Self::role_to_string(&published.role));
        }
        if let Some(created_at) = update.created_at {
            sql.push(", created_at = ").push_bind(created_at);
        }
        if let Some(updated_at) = update.updated_at {
            sql.push(", updated_at = ").push_bind(updated_at);
        }
        if let Some(classes_ids) = &update.classes_ids {
            let class_id = classes_ids
                .as_ref()
                .and_then(|ids| ids.first())
                .map(|id| id.to_hex());
            sql.push(", class_id = ").push_bind(class_id);
        }

        sql.push(" WHERE id = ")
            .push_bind(&id)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        if let Some(classes_ids) = &update.classes_ids {
            self.replace_classes(&id, classes_ids.as_ref()).await?;
        }
        if let Some(mention) = &update.mention {
            self.replace_mentions(&id, mention.as_ref()).await?;
        }

        self.find_one(Some(&IdType::String(id)), None, None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Announcement, AppError> {
        let announcement = self.find_one(Some(id), None, None).await?;
        sqlx::query(
            "UPDATE announcements SET deleted_at = now(), updated_at = now() WHERE id = $1",
        )
        .bind(Self::id_to_string(id)?)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(announcement)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
        school_id: Option<&str>,
    ) -> Result<Paginated<AnnouncementWithRelations>, AppError> {
        let page = self.get_all(filter, limit, skip, query, school_id).await?;
        Ok(Paginated {
            data: page
                .data
                .into_iter()
                .map(|announcement| AnnouncementWithRelations {
                    announcement,
                    published_user: None,
                    mentioned_users: None,
                    classes: None,
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
    ) -> Result<AnnouncementWithRelations, AppError> {
        let announcement = self.find_one(id, query, school_id).await?;
        Ok(AnnouncementWithRelations {
            announcement,
            published_user: None,
            mentioned_users: None,
            classes: None,
        })
    }

    pub async fn count_announcements(
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
}

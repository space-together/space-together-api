use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::{Paginated, SubjectCategory},
        template_subject::{TemplateSubject, TemplateSubjectPartial, TemplateSubjectWithOthers},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    utils::object_id::ObjectId,
};

pub struct TemplateSubjectService {
    pub pool: PgPool,
}

impl TemplateSubjectService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError { message: format!("PostgreSQL Error: {}", err) }
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

    fn category_to_string(c: &SubjectCategory) -> String {
        c.clone().into()
    }

    fn category_from_string(s: Option<String>) -> SubjectCategory {
        let raw = s.unwrap_or_else(|| "Other".to_string());
        serde_json::from_value(serde_json::Value::String(raw.clone()))
            .unwrap_or(SubjectCategory::Other(raw))
    }

    fn row_to_subject(row: PgRow) -> Result<TemplateSubject, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let created_by_str: Option<String> = row.try_get("created_by").ok().flatten();
        let prereqs_json: serde_json::Value = row.try_get("prerequisites").unwrap_or(serde_json::Value::Array(vec![]));
        let topics_json: serde_json::Value = row.try_get("topics").unwrap_or(serde_json::Value::Array(vec![]));
        let prerequisites = serde_json::from_value(prereqs_json).ok();
        let topics = serde_json::from_value(topics_json).ok();

        Ok(TemplateSubject {
            id: Some(Self::parse_oid(&id, "id")?),
            name: row.try_get("name").map_err(Self::db_error)?,
            code: row.try_get("code").map_err(Self::db_error)?,
            description: row.try_get("description").map_err(Self::db_error)?,
            category: Self::category_from_string(row.try_get("category").ok().flatten()),
            estimated_hours: row.try_get("estimated_hours").unwrap_or(0),
            credits: row.try_get("credits").ok().flatten(),
            prerequisites,
            topics,
            created_by: created_by_str.as_deref().map(|s| Self::parse_oid(s, "created_by")).transpose()?,
            disable: row.try_get("disable").ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    fn select_sql() -> &'static str {
        "SELECT id, name, code, description, category, estimated_hours, credits, \
         prerequisites, topics, created_by, disable, created_at, updated_at \
         FROM template_subjects WHERE 1=1"
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str) {
        sql.push(" AND (lower(name) LIKE ").push_bind(like)
            .push(" OR lower(code) LIKE ").push_bind(like)
            .push(" OR lower(coalesce(description,'')) LIKE ").push_bind(like)
            .push(" OR lower(category) LIKE ").push_bind(like)
            .push(" OR lower(id) LIKE ").push_bind(like)
            .push(")");
    }

    fn push_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RequestQuery>,
    ) -> Result<(), AppError> {
        let Some(q) = query else { return Ok(()) };

        if !q.by_ids.is_empty() {
            sql.push(" AND id IN (");
            let mut sep = sql.separated(", ");
            for id in &q.by_ids { sep.push_bind(id); }
            sep.push_unseparated(")");
        }

        if q.field.len() != q.value.len() {
            return Err(AppError { message: "Number of fields must match values".into() });
        }
        for (field, value) in q.field.iter().zip(q.value.iter()) {
            match field.as_str() {
                "id" | "_id" => { sql.push(" AND id = ").push_bind(value); }
                "code" => { sql.push(" AND lower(code) = lower(").push_bind(value).push(")"); }
                "name" => { sql.push(" AND lower(name) = lower(").push_bind(value).push(")"); }
                "category" => { sql.push(" AND lower(category) = lower(").push_bind(value).push(")"); }
                "disable" => {
                    let b = value.parse::<bool>().map_err(|_| AppError { message: "Invalid disable".into() })?;
                    sql.push(" AND disable = ").push_bind(b);
                }
                _ => return Err(AppError { message: format!("Unsupported filter: {}", field) }),
            }
        }
        Ok(())
    }

    pub async fn create(&self, dto: TemplateSubject) -> Result<TemplateSubject, AppError> {
        if self.find_one_by_code(&dto.code).await.is_ok() {
            return Err(AppError {
                message: format!("Subject code is ready exit try other not {}", dto.code),
            });
        }

        let id = Self::new_id();
        let now = Utc::now();
        let prerequisites_json = serde_json::to_value(&dto.prerequisites).unwrap_or(serde_json::Value::Array(vec![]));
        let topics_json = serde_json::to_value(&dto.topics).unwrap_or(serde_json::Value::Array(vec![]));

        sqlx::query(
            r#"INSERT INTO template_subjects (id, name, code, description, category, estimated_hours,
               credits, prerequisites, topics, created_by, disable, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"#,
        )
        .bind(&id)
        .bind(&dto.name)
        .bind(&dto.code)
        .bind(&dto.description)
        .bind(Self::category_to_string(&dto.category))
        .bind(dto.estimated_hours)
        .bind(dto.credits)
        .bind(&prerequisites_json)
        .bind(&topics_json)
        .bind(dto.created_by.as_ref().map(|o| o.to_hex()))
        .bind(dto.disable.unwrap_or(false))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one_by_id(&IdType::from_string(id)).await
    }

    pub async fn find_one_by_id(&self, id: &IdType) -> Result<TemplateSubject, AppError> {
        let id_str = Self::id_to_string(id)?;
        let row = sqlx::query(&format!("{} AND id = $1", Self::select_sql()))
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_subject(r),
            None => Err(AppError { message: "Template subject not found".to_string() }),
        }
    }

    pub async fn find_one_by_code(&self, code: &str) -> Result<TemplateSubject, AppError> {
        let row = sqlx::query(&format!("{} AND lower(code) = lower($1)", Self::select_sql()))
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_subject(r),
            None => Err(AppError { message: "Template subject not found by code".to_string() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
    ) -> Result<Paginated<TemplateSubject>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter.as_ref().filter(|v| !v.trim().is_empty()).map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM template_subjects WHERE 1=1");
        Self::push_filters(&mut cq, query)?;
        if let Some(ref l) = like { Self::push_search(&mut cq, l); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_filters(&mut dq, query)?;
        if let Some(ref l) = like { Self::push_search(&mut dq, l); }
        dq.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit).push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_subject).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn update_subject(
        &self,
        id: &IdType,
        update: &TemplateSubjectPartial,
    ) -> Result<TemplateSubject, AppError> {
        if let Some(code) = update.code.clone() {
            if let Ok(sub) = self.find_one_by_code(&code).await {
                let existing_id = sub.id.as_ref().map(|o| o.to_hex());
                let target_id = Some(Self::id_to_string(id)?);
                if existing_id != target_id {
                    return Err(AppError { message: format!("Subject code is ready exit try other not {}", sub.code) });
                }
            }
        }

        let id_str = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE template_subjects SET updated_at = now()");
        let mut touched = false;

        if let Some(v) = &update.name { sql.push(", name = ").push_bind(v); touched = true; }
        if let Some(v) = &update.code { sql.push(", code = ").push_bind(v); touched = true; }
        if let Some(v) = &update.description { sql.push(", description = ").push_bind(v); touched = true; }
        if let Some(ref v) = update.category {
            sql.push(", category = ").push_bind(Self::category_to_string(v));
            touched = true;
        }
        if let Some(v) = update.estimated_hours { sql.push(", estimated_hours = ").push_bind(v); touched = true; }
        if let Some(v) = update.credits { sql.push(", credits = ").push_bind(v); touched = true; }
        if let Some(v) = update.disable { sql.push(", disable = ").push_bind(v); touched = true; }
        if let Some(ref v) = update.prerequisites {
            let j = serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]));
            sql.push(", prerequisites = ").push_bind(j);
            touched = true;
        }
        if let Some(ref v) = update.topics {
            let j = serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]));
            sql.push(", topics = ").push_bind(j);
            touched = true;
        }

        if !touched { return Err(AppError { message: "No valid fields to update".into() }); }

        sql.push(" WHERE id = ").push_bind(&id_str);
        sql.build().execute(&self.pool).await.map_err(Self::db_error)?;

        self.find_one_by_id(id).await
    }

    pub async fn delete_subject(&self, id: &IdType) -> Result<TemplateSubject, AppError> {
        let sub = self.find_one_by_id(id).await?;
        sqlx::query("DELETE FROM template_subjects WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(sub)
    }

    async fn build_with_others(&self, subject: TemplateSubject) -> Result<TemplateSubjectWithOthers, AppError> {
        // Fetch creator user
        let creator_user = if let Some(ref created_by) = subject.created_by {
            let user_row = sqlx::query(
                "SELECT id, name, email, username, role, image, image_id, phone, gender, \
                 disabled, current_school_id, current_school_user_id, schools, accessible_classes, \
                 raw_document, created_at, updated_at, deleted_at FROM users WHERE id = $1"
            )
            .bind(created_by.to_hex())
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
            user_row.and_then(|r| {
                let raw: serde_json::Value = r.try_get("raw_document").unwrap_or(serde_json::Value::Object(Default::default()));
                serde_json::from_value(raw).ok()
            })
        } else {
            None
        };

        // Fetch prerequisite main classes
        let prerequisite_classes = if let Some(ref prereqs) = subject.prerequisites {
            if prereqs.is_empty() {
                Some(vec![])
            } else {
                let ids: Vec<String> = prereqs.iter().map(|o| o.to_hex()).collect();
                let mut sql = QueryBuilder::<Postgres>::new(
                    "SELECT id, name, username, trade_id, level, description, disable, created_at, updated_at \
                     FROM main_classes WHERE id IN (",
                );
                let mut sep = sql.separated(", ");
                for id in &ids { sep.push_bind(id); }
                sep.push_unseparated(")");
                let rows = sql.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
                let classes = rows.into_iter().map(|r| crate::services::main_class_service::MainClassService::row_to_main_class_pub(r)).collect::<Result<Vec<_>, _>>()?;
                Some(classes)
            }
        } else {
            Some(vec![])
        };

        Ok(TemplateSubjectWithOthers { subject, creator_user, prerequisite_classes })
    }

    pub async fn get_all_with_other(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<TemplateSubjectWithOthers>, AppError> {
        let result = self.get_all(filter, limit, skip, None).await?;
        let mut data = Vec::with_capacity(result.data.len());
        for sub in result.data {
            data.push(self.build_with_others(sub).await?);
        }
        Ok(Paginated { data, total: result.total, total_pages: result.total_pages, current_page: result.current_page })
    }

    pub async fn find_one_with_relations(&self, id: &IdType) -> Result<TemplateSubjectWithOthers, AppError> {
        let sub = self.find_one_by_id(id).await?;
        self.build_with_others(sub).await
    }

    pub async fn find_one_with_relations_by_code(&self, code: &str) -> Result<TemplateSubjectWithOthers, AppError> {
        let sub = self.find_one_by_code(code).await?;
        self.build_with_others(sub).await
    }

    pub async fn find_many_by_prerequisite_with_relations(
        &self,
        prerequisite_id: &IdType,
    ) -> Result<Vec<TemplateSubjectWithOthers>, AppError> {
        let prereq_str = Self::id_to_string(prerequisite_id)?;
        let result = self.get_all_where_prerequisite(&prereq_str).await?;
        let mut data = Vec::with_capacity(result.len());
        for sub in result {
            data.push(self.build_with_others(sub).await?);
        }
        Ok(data)
    }

    pub async fn find_many_by_prerequisite(
        &self,
        prerequisite_id: &IdType,
    ) -> Result<Vec<TemplateSubject>, AppError> {
        let prereq_str = Self::id_to_string(prerequisite_id)?;
        self.get_all_where_prerequisite(&prereq_str).await
    }

    async fn get_all_where_prerequisite(&self, prereq_id: &str) -> Result<Vec<TemplateSubject>, AppError> {
        let rows = sqlx::query(&format!(
            "{} AND prerequisites @> $1::jsonb",
            Self::select_sql()
        ))
        .bind(format!("[\"{}\"]", prereq_id))
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;
        rows.into_iter().map(Self::row_to_subject).collect()
    }

    pub async fn count_template_subjects(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
    ) -> Result<CountDoc, AppError> {
        let total = self.get_all(filter, Some(1), Some(0), query).await?.total;
        Ok(CountDoc { count: total as u64 })
    }
}

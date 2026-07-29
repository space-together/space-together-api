use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        score::{Score, ScoreAuditLog, ScorePartial},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    utils::object_id::{parse_object_id_value, ObjectId},
};

#[derive(Debug, Clone, Default)]
pub struct ScoreQuery {
    pub by_ids: Vec<String>,
    pub school_id: Option<String>,
    pub student_id: Option<String>,
    pub class_subject_id: Option<String>,
    pub exam_id: Option<String>,
    pub education_year_id: Option<String>,
    pub field_values: Vec<(String, String)>,
}

impl ScoreQuery {
    pub fn from_request(
        query: &RequestQuery,
        context_school_id: Option<String>,
    ) -> Result<Self, AppError> {
        if query.field.len() != query.value.len() {
            return Err(AppError {
                message: "Number of fields must match number of values".into(),
            });
        }
        for id in &query.by_ids {
            parse_object_id_value(id)?;
        }

        Ok(Self {
            by_ids: query.by_ids.clone(),
            school_id: query.school_id.clone().or(context_school_id),
            student_id: None,
            class_subject_id: None,
            exam_id: query.exam_id.clone(),
            education_year_id: query.education_year_id.clone(),
            field_values: query
                .field
                .iter()
                .cloned()
                .zip(query.value.iter().cloned())
                .collect(),
        })
    }

    pub fn from_school_context(context_school_id: Option<String>) -> Self {
        Self {
            school_id: context_school_id,
            ..Self::default()
        }
    }
}

pub struct ScoreService {
    pub pool: PgPool,
}

impl ScoreService {
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

    fn select_sql() -> &'static str {
        r#"
        SELECT id, school_id, student_id, class_subject_id, exam_id,
               assessment_category_id, education_year_id,
               score::DOUBLE PRECISION AS score,
               max_score::DOUBLE PRECISION AS max_score,
               percentage::DOUBLE PRECISION AS percentage,
               remarks, COALESCE(entered_by, recorded_by) AS entered_by,
               created_at, updated_at, deleted_at
        FROM scores
        WHERE deleted_at IS NULL
        "#
    }

    fn push_query_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a ScoreQuery>,
    ) -> Result<(), AppError> {
        let Some(query) = query else {
            return Ok(());
        };

        if !query.by_ids.is_empty() {
            sql.push(" AND id IN (");
            let mut separated = sql.separated(", ");
            for id in &query.by_ids {
                parse_object_id_value(id)?;
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
        }

        for (column, value) in [
            ("school_id", query.school_id.as_ref()),
            ("student_id", query.student_id.as_ref()),
            ("class_subject_id", query.class_subject_id.as_ref()),
            ("exam_id", query.exam_id.as_ref()),
            ("education_year_id", query.education_year_id.as_ref()),
        ] {
            if let Some(value) = value {
                parse_object_id_value(value)?;
                sql.push(" AND ").push(column).push(" = ").push_bind(value);
            }
        }

        for (field, value) in &query.field_values {
            match field.as_str() {
                "_id" | "id" => {
                    parse_object_id_value(value)?;
                    sql.push(" AND id = ").push_bind(value);
                }
                "school_id"
                | "student_id"
                | "class_subject_id"
                | "exam_id"
                | "assessment_category_id"
                | "education_year_id"
                | "entered_by" => {
                    parse_object_id_value(value)?;
                    if field == "entered_by" {
                        sql.push(" AND COALESCE(entered_by, recorded_by) = ")
                            .push_bind(value);
                    } else {
                        sql.push(" AND ").push(field).push(" = ").push_bind(value);
                    }
                }
                _ => {
                    return Err(AppError {
                        message: format!("Unsupported score filter field: {}", field),
                    });
                }
            }
        }
        Ok(())
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, search_like: &'a str) {
        sql.push(" AND (lower(id) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(remarks, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(school_id, '')) LIKE ")
            .push_bind(search_like)
            .push(" OR lower(coalesce(student_id, '')) LIKE ")
            .push_bind(search_like)
            .push(")");
    }

    fn score_from_row(row: PgRow) -> Result<Score, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Score {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid_opt(row.try_get("school_id").ok().flatten(), "school_id")?,
            student_id: Self::parse_oid_opt(
                row.try_get("student_id").ok().flatten(),
                "student_id",
            )?,
            class_subject_id: Self::parse_oid_opt(
                row.try_get("class_subject_id").ok().flatten(),
                "class_subject_id",
            )?,
            exam_id: Self::parse_oid_opt(row.try_get("exam_id").ok().flatten(), "exam_id")?,
            assessment_category_id: Self::parse_oid_opt(
                row.try_get("assessment_category_id").ok().flatten(),
                "assessment_category_id",
            )?,
            education_year_id: Self::parse_oid_opt(
                row.try_get("education_year_id").ok().flatten(),
                "education_year_id",
            )?,
            score: row.try_get("score").map_err(Self::db_error)?,
            max_score: row.try_get("max_score").map_err(Self::db_error)?,
            percentage: row.try_get("percentage").map_err(Self::db_error)?,
            remarks: row.try_get("remarks").ok().flatten(),
            entered_by: Self::parse_oid_opt(
                row.try_get("entered_by").ok().flatten(),
                "entered_by",
            )?,
            created_at: row.try_get("created_at").ok().flatten(),
            updated_at: row.try_get("updated_at").ok().flatten(),
            is_deleted: row
                .try_get::<Option<DateTime<Utc>>, _>("deleted_at")
                .ok()
                .flatten()
                .is_some(),
        })
    }

    fn audit_from_row(row: PgRow) -> Result<ScoreAuditLog, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(ScoreAuditLog {
            id: Some(Self::parse_oid(&id, "id")?),
            score_id: Self::parse_oid_opt(row.try_get("score_id").ok().flatten(), "score_id")?,
            old_score: row.try_get("old_score").map_err(Self::db_error)?,
            new_score: row.try_get("new_score").map_err(Self::db_error)?,
            changed_by: Self::parse_oid_opt(
                row.try_get("changed_by").ok().flatten(),
                "changed_by",
            )?,
            change_reason: row.try_get("change_reason").ok().flatten(),
            changed_at: row.try_get("changed_at").ok().flatten(),
        })
    }

    pub async fn create(&self, mut score: Score) -> Result<Score, AppError> {
        self.ensure_indexes().await?;
        score.percentage = if score.max_score > 0.0 {
            (score.score / score.max_score) * 100.0
        } else {
            0.0
        };
        if score.score > score.max_score {
            return Err(AppError {
                message: "Score cannot exceed maximum score".to_string(),
            });
        }

        let school_id = score
            .school_id
            .as_ref()
            .map(|id| id.to_hex())
            .ok_or_else(|| AppError {
                message: "school_id is required".into(),
            })?;
        let id = score.id.map(|id| id.to_hex()).unwrap_or_else(Self::new_id);

        let result = sqlx::query(
            r#"
            INSERT INTO scores (
              id, school_id, student_id, class_subject_id, exam_id, assessment_category_id,
              education_year_id, score, max_score, percentage, remarks,
              entered_by, recorded_by, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12, COALESCE($13, now()), COALESCE($14, now()), $15)
            "#,
        )
        .bind(&id)
        .bind(&school_id)
        .bind(score.student_id.as_ref().map(|id| id.to_hex()))
        .bind(score.class_subject_id.as_ref().map(|id| id.to_hex()))
        .bind(score.exam_id.as_ref().map(|id| id.to_hex()))
        .bind(score.assessment_category_id.as_ref().map(|id| id.to_hex()))
        .bind(score.education_year_id.as_ref().map(|id| id.to_hex()))
        .bind(score.score)
        .bind(score.max_score)
        .bind(score.percentage)
        .bind(&score.remarks)
        .bind(score.entered_by.as_ref().map(|id| id.to_hex()))
        .bind(score.created_at)
        .bind(score.updated_at)
        .bind(if score.is_deleted { Some(Utc::now()) } else { None })
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => self.find_one(&IdType::from_string(id)).await,
            Err(error)
                if error
                    .as_database_error()
                    .map(|error| error.is_unique_violation())
                    .unwrap_or(false) =>
            {
                Err(AppError {
                    message:
                        "Score already exists for this student, subject, exam, and assessment category"
                            .to_string(),
                })
            }
            Err(error) => Err(Self::db_error(error)),
        }
    }

    pub async fn create_many(&self, scores: Vec<Score>) -> Result<Vec<Score>, AppError> {
        let mut created_scores = Vec::new();
        for score in scores {
            match self.create(score).await {
                Ok(score) => created_scores.push(score),
                Err(error) => eprintln!("Failed to create score: {}", error.message),
            }
        }
        Ok(created_scores)
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Score, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        sql.push(" ORDER BY updated_at DESC LIMIT 1");
        let row = sql
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        row.map(Self::score_from_row).transpose()?.ok_or(AppError {
            message: "Score not found".into(),
        })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<ScoreQuery>,
    ) -> Result<Paginated<Score>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let search_like = filter
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("%{}%", value.to_lowercase()));

        let mut count_query =
            QueryBuilder::<Postgres>::new("SELECT count(*) FROM scores WHERE deleted_at IS NULL");
        Self::push_query_filters(&mut count_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_search(&mut count_query, search_like);
        }
        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        let mut data_query = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_query_filters(&mut data_query, query.as_ref())?;
        if let Some(search_like) = search_like.as_deref() {
            Self::push_search(&mut data_query, search_like);
        }
        data_query
            .push(" ORDER BY updated_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(skip);
        let rows = data_query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(Self::db_error)?;
        let data = rows
            .into_iter()
            .map(Self::score_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(
        &self,
        id: &IdType,
        update: &ScorePartial,
        changed_by: &ObjectId,
        change_reason: Option<String>,
    ) -> Result<Score, AppError> {
        let existing = self.find_one(id).await?;
        let mut new_score_value = existing.score;
        let next_score = update.score.unwrap_or(existing.score);
        let next_max_score = update.max_score.unwrap_or(existing.max_score);
        if next_score > next_max_score {
            return Err(AppError {
                message: "Score cannot exceed maximum score".to_string(),
            });
        }
        let next_percentage = if next_max_score > 0.0 {
            (next_score / next_max_score) * 100.0
        } else {
            0.0
        };

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE scores SET updated_at = now()");
        let mut touched = false;
        if update.score.is_some() {
            sql.push(", score = ")
                .push_bind(next_score)
                .push(", percentage = ")
                .push_bind(next_percentage);
            new_score_value = next_score;
            touched = true;
        }
        if update.max_score.is_some() {
            sql.push(", max_score = ")
                .push_bind(next_max_score)
                .push(", percentage = ")
                .push_bind(next_percentage);
            touched = true;
        }
        if let Some(value) = update.remarks.clone() {
            sql.push(", remarks = ").push_bind(value);
            touched = true;
        }
        for (field, value) in [
            ("school_id", update.school_id),
            ("student_id", update.student_id),
            ("class_subject_id", update.class_subject_id),
            ("exam_id", update.exam_id),
            ("assessment_category_id", update.assessment_category_id),
            ("education_year_id", update.education_year_id),
            ("entered_by", update.entered_by),
        ] {
            if let Some(value) = value {
                let value = value.map(|id| id.to_hex());
                if field == "entered_by" {
                    sql.push(", entered_by = ")
                        .push_bind(value.clone())
                        .push(", recorded_by = ")
                        .push_bind(value);
                } else {
                    sql.push(", ").push(field).push(" = ").push_bind(value);
                }
                touched = true;
            }
        }
        if let Some(value) = update.is_deleted {
            sql.push(", deleted_at = ");
            if value {
                sql.push("now()");
            } else {
                sql.push("NULL");
            }
            touched = true;
        }
        if !touched {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        if new_score_value != existing.score {
            self.insert_audit_log(
                existing.id,
                existing.school_id,
                existing.score,
                new_score_value,
                *changed_by,
                change_reason,
            )
            .await?;
        }

        sql.push(" WHERE id = ")
            .push_bind(Self::id_to_string(id)?)
            .push(" AND deleted_at IS NULL");
        sql.build()
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        self.find_one(id).await
    }

    async fn insert_audit_log(
        &self,
        score_id: Option<ObjectId>,
        school_id: Option<ObjectId>,
        old_score: f64,
        new_score: f64,
        changed_by: ObjectId,
        change_reason: Option<String>,
    ) -> Result<(), AppError> {
        let score_id = score_id.ok_or_else(|| AppError {
            message: "Score has no ID".into(),
        })?;
        let school_id = school_id.ok_or_else(|| AppError {
            message: "Score has no school_id".into(),
        })?;
        sqlx::query(
            r#"
            INSERT INTO score_audit_logs (
              id, school_id, score_id, changed_by, old_score, new_score, reason, change_reason, created_at, changed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, now(), now())
            "#,
        )
        .bind(Self::new_id())
        .bind(school_id.to_hex())
        .bind(score_id.to_hex())
        .bind(changed_by.to_hex())
        .bind(old_score)
        .bind(new_score)
        .bind(change_reason)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;
        Ok(())
    }

    pub async fn delete(&self, id: &IdType) -> Result<Score, AppError> {
        let score = self.find_one(id).await?;
        sqlx::query("UPDATE scores SET deleted_at = now(), updated_at = now() WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(score)
    }

    pub async fn get_student_exam_scores(
        &self,
        student_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Vec<Score>, AppError> {
        let query = ScoreQuery {
            student_id: Some(student_id.to_hex()),
            exam_id: Some(exam_id.to_hex()),
            ..ScoreQuery::default()
        };
        Ok(self
            .get_all(None, Some(500), Some(0), Some(query))
            .await?
            .data)
    }

    pub async fn get_audit_logs(
        &self,
        score_id: &ObjectId,
    ) -> Result<Vec<ScoreAuditLog>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, score_id, old_score::DOUBLE PRECISION AS old_score,
                   new_score::DOUBLE PRECISION AS new_score, changed_by,
                   COALESCE(change_reason, reason) AS change_reason,
                   COALESCE(changed_at, created_at) AS changed_at
            FROM score_audit_logs
            WHERE score_id = $1
            ORDER BY COALESCE(changed_at, created_at) DESC
            "#,
        )
        .bind(score_id.to_hex())
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        rows.into_iter().map(Self::audit_from_row).collect()
    }
}

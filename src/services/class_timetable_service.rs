use actix_web::web;
use chrono::Weekday;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    config::state::AppState,
    domain::{
        class_timetable::{ClassTimetable, ClassTimetablePartial, DayStructureConfig},
        common_details::Paginated,
    },
    errors::AppError,
    handler::class_timetable_handler::auto_generate_schedule,
    models::{id_model::IdType, school_token_model::SchoolToken},
    services::{
        class_subject_service::{ClassSubjectQuery, ClassSubjectService},
        education_year_service::{EducationYearQuery, EducationYearService},
    },
    utils::object_id::ObjectId,
};

pub struct ClassTimetableService {
    pub pool: PgPool,
}

impl ClassTimetableService {
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

    fn row_to_timetable(row: PgRow) -> Result<ClassTimetable, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let class_id_str: String = row.try_get("class_id").map_err(Self::db_error)?;
        let edu_year_str: String = row.try_get("education_year_id").map_err(Self::db_error)?;
        let weekly_json: serde_json::Value = row.try_get("weekly_schedule").unwrap_or(serde_json::Value::Array(vec![]));

        Ok(ClassTimetable {
            id: Some(Self::parse_oid(&id, "id")?),
            class_id: Self::parse_oid(&class_id_str, "class_id")?,
            education_year_id: Self::parse_oid(&edu_year_str, "education_year_id")?,
            term_order: row.try_get("term_order").unwrap_or(1),
            weekly_schedule: serde_json::from_value(weekly_json).unwrap_or_default(),
            disabled: row.try_get("disabled").ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    fn select_sql() -> &'static str {
        "SELECT id, class_id, education_year_id, term_order, weekly_schedule, disabled, \
         created_at, updated_at FROM class_timetables WHERE 1=1"
    }

    pub async fn create(&self, dto: ClassTimetable) -> Result<ClassTimetable, AppError> {
        if self.find_by_class_and_year(&dto.class_id, &dto.education_year_id).await.is_ok() {
            return Err(AppError {
                message: format!(
                    "Timetable for class {} in year {} already exists",
                    dto.class_id, dto.education_year_id
                ),
            });
        }

        for week_schedule in &dto.weekly_schedule {
            if let Err(e) = week_schedule.validate() {
                return Err(AppError { message: format!("Validation error on {}: {}", week_schedule.day, e) });
            }
        }

        let id = Self::new_id();
        let class_id_str = dto.class_id.to_hex();
        let edu_year_str = dto.education_year_id.to_hex();
        let weekly_json = serde_json::to_value(&dto.weekly_schedule).unwrap_or(serde_json::Value::Array(vec![]));

        sqlx::query(
            r#"INSERT INTO class_timetables (id, class_id, education_year_id, term_order, weekly_schedule, disabled, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,now(),now())"#,
        )
        .bind(&id)
        .bind(&class_id_str)
        .bind(&edu_year_str)
        .bind(dto.term_order)
        .bind(&weekly_json)
        .bind(dto.disabled.unwrap_or(false))
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one_by_id(&IdType::from_string(id)).await
    }

    pub async fn find_one_by_id(&self, id: &IdType) -> Result<ClassTimetable, AppError> {
        let id_str = Self::id_to_string(id)?;
        let row = sqlx::query(&format!("{} AND id = $1", Self::select_sql()))
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_timetable(r),
            None => Err(AppError { message: "Class timetable not found".to_string() }),
        }
    }

    pub async fn find_by_class_and_year(
        &self,
        class_id: &ObjectId,
        education_year_id: &ObjectId,
    ) -> Result<ClassTimetable, AppError> {
        let row = sqlx::query(&format!("{} AND class_id = $1 AND education_year_id = $2 LIMIT 1", Self::select_sql()))
            .bind(class_id.to_hex())
            .bind(education_year_id.to_hex())
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_timetable(r),
            None => Err(AppError { message: "Class timetable not found for this class/year".to_string() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<ClassTimetable>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM class_timetables WHERE 1=1");
        if let Some(ref f) = filter {
            if !f.trim().is_empty() {
                let like = format!("%{}%", f.to_lowercase());
                cq.push(" AND lower(class_id) LIKE ").push_bind(like);
            }
        }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(ref f) = filter {
            if !f.trim().is_empty() {
                let like = format!("%{}%", f.to_lowercase());
                dq.push(" AND lower(class_id) LIKE ").push_bind(like);
            }
        }
        dq.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit).push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_timetable).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn update_timetable(
        &self,
        id: &IdType,
        dto: &ClassTimetablePartial,
    ) -> Result<ClassTimetable, AppError> {
        let id_str = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE class_timetables SET updated_at = now()");
        let mut touched = false;

        if let Some(v) = &dto.class_id { sql.push(", class_id = ").push_bind(v.to_hex()); touched = true; }
        if let Some(v) = &dto.education_year_id { sql.push(", education_year_id = ").push_bind(v.to_hex()); touched = true; }
        if let Some(v) = dto.term_order { sql.push(", term_order = ").push_bind(v); touched = true; }
        if let Some(v) = dto.disabled { sql.push(", disabled = ").push_bind(v); touched = true; }
        if let Some(ref v) = dto.weekly_schedule {
            let j = serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]));
            sql.push(", weekly_schedule = ").push_bind(j); touched = true;
        }

        if !touched { return Err(AppError { message: "No valid fields to update".into() }); }

        sql.push(" WHERE id = ").push_bind(&id_str);
        sql.build().execute(&self.pool).await.map_err(Self::db_error)?;

        self.find_one_by_id(id).await
    }

    pub async fn delete_timetable(&self, id: &IdType) -> Result<ClassTimetable, AppError> {
        let existing = self.find_one_by_id(id).await?;
        sqlx::query("DELETE FROM class_timetables WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(existing)
    }

    pub async fn generate_timetable(
        &self,
        class_id: &IdType,
        state: &web::Data<AppState>,
        school_claims: &Option<SchoolToken>,
    ) -> Result<ClassTimetable, AppError> {
        let education_service = EducationYearService::new(&state.pg.pool);
        let subject_service = ClassSubjectService::new(&state.pg.pool);

        let (education_year, term_info) = education_service
            .get_current_year_and_term(None, Some(EducationYearQuery::from_school_context(school_claims.as_ref().map(|c| c.id.clone()))))
            .await?;

        let term_order = term_info.map(|t| t.order).unwrap_or(1);

        let class_subjects = subject_service
            .get_all(None, None, None, Some(ClassSubjectQuery {
                school_id: school_claims.as_ref().map(|c| c.id.clone()),
                class_id: Some(IdType::to_object_id(class_id)?.to_hex()),
                ..ClassSubjectQuery::default()
            }))
            .await?;

        let class_id_obj = IdType::to_object_id(class_id)?;
        let education_year_id = education_year.id.unwrap();
        let days = vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri];
        let day_template = DayStructureConfig::standard_day();

        let timetable = auto_generate_schedule(
            class_id_obj,
            education_year_id,
            term_order,
            class_subjects.data,
            "08:00".to_string(),
            days,
            day_template,
        )
        .map_err(|err| AppError { message: err })?;

        self.create(timetable).await
    }
}

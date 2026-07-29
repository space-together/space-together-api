use chrono::Weekday;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        school_timetable::{
            DailySchoolSchedule, DaySpecialType, SchoolTimetable, SchoolTimetablePartial, TimeBlock,
        },
    },
    errors::AppError,
    models::id_model::IdType,
    utils::object_id::ObjectId,
};

pub struct SchoolTimetableService {
    pub pool: PgPool,
}

impl SchoolTimetableService {
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

    fn row_to_timetable(row: PgRow) -> Result<SchoolTimetable, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let school_id_str: String = row.try_get("school_id").map_err(Self::db_error)?;
        let academic_year_id_str: Option<String> = row.try_get("academic_year_id").ok().flatten();

        let weekly_json: serde_json::Value = row.try_get("default_weekly_schedule").unwrap_or(serde_json::Value::Array(vec![]));
        let overrides_json: serde_json::Value = row.try_get("overrides").unwrap_or(serde_json::Value::Array(vec![]));
        let events_json: serde_json::Value = row.try_get("events").unwrap_or(serde_json::Value::Array(vec![]));

        Ok(SchoolTimetable {
            id: Some(Self::parse_oid(&id, "id")?),
            school_id: Self::parse_oid(&school_id_str, "school_id")?,
            academic_year_id: academic_year_id_str.as_deref().map(|s| Self::parse_oid(s, "academic_year_id")).transpose()?,
            default_weekly_schedule: serde_json::from_value(weekly_json).unwrap_or_default(),
            overrides: serde_json::from_value(overrides_json).ok(),
            events: serde_json::from_value(events_json).ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    fn select_sql() -> &'static str {
        "SELECT id, school_id, academic_year_id, default_weekly_schedule, overrides, events, \
         created_at, updated_at FROM school_timetables WHERE 1=1"
    }

    pub async fn create(&self, dto: SchoolTimetable) -> Result<SchoolTimetable, AppError> {
        if self.find_by_school_and_academic_year(&dto.school_id, &dto.academic_year_id).await.is_ok() {
            return Err(AppError { message: "School timetable for academic year already exists".to_string() });
        }

        if let Err(e) = dto.validate() {
            return Err(AppError { message: format!("Validation error: {}", e) });
        }

        let id = Self::new_id();
        let school_id_str = dto.school_id.to_hex();
        let academic_year_id_str = dto.academic_year_id.as_ref().map(|o| o.to_hex());
        let weekly_json = serde_json::to_value(&dto.default_weekly_schedule).unwrap_or(serde_json::Value::Array(vec![]));
        let overrides_json = serde_json::to_value(&dto.overrides).unwrap_or(serde_json::Value::Array(vec![]));
        let events_json = serde_json::to_value(&dto.events).unwrap_or(serde_json::Value::Array(vec![]));

        sqlx::query(
            r#"INSERT INTO school_timetables (id, school_id, academic_year_id, default_weekly_schedule, overrides, events, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,now(),now())"#,
        )
        .bind(&id)
        .bind(&school_id_str)
        .bind(&academic_year_id_str)
        .bind(&weekly_json)
        .bind(&overrides_json)
        .bind(&events_json)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_school_id: Option<&str>,
    ) -> Result<SchoolTimetable, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        if let Some(sid) = extra_school_id {
            sql.push(" AND school_id = ").push_bind(sid);
        }
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_timetable(r),
            None => Err(AppError { message: "School timetable not found".to_string() }),
        }
    }

    pub async fn find_by_school_and_academic_year(
        &self,
        school_id: &ObjectId,
        academic_year_id: &Option<ObjectId>,
    ) -> Result<SchoolTimetable, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        sql.push(" AND school_id = ").push_bind(school_id.to_hex());
        match academic_year_id {
            Some(id) => { sql.push(" AND academic_year_id = ").push_bind(id.to_hex()); }
            None => { sql.push(" AND academic_year_id IS NULL"); }
        }
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_timetable(r),
            None => Err(AppError { message: "School timetable not found for this academic year".into() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<SchoolTimetable>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM school_timetables WHERE 1=1");
        if let Some(ref f) = filter {
            if !f.trim().is_empty() {
                let like = format!("%{}%", f.to_lowercase());
                cq.push(" AND lower(school_id) LIKE ").push_bind(like);
            }
        }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(ref f) = filter {
            if !f.trim().is_empty() {
                let like = format!("%{}%", f.to_lowercase());
                dq.push(" AND lower(school_id) LIKE ").push_bind(like);
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
        dto: &SchoolTimetablePartial,
    ) -> Result<SchoolTimetable, AppError> {
        let obj_id_str = Self::id_to_string(id)?;

        // Uniqueness conflict check
        if let (Some(sid), Some(ayid)) = (dto.school_id.as_ref(), dto.academic_year_id.as_ref()) {
            if let Some(existing_ay) = ayid {
                let row = sqlx::query(
                    "SELECT id FROM school_timetables WHERE school_id = $1 AND academic_year_id = $2 AND id <> $3 LIMIT 1"
                )
                .bind(sid.to_hex())
                .bind(existing_ay.to_hex())
                .bind(&obj_id_str)
                .fetch_optional(&self.pool)
                .await
                .map_err(Self::db_error)?;
                if row.is_some() {
                    return Err(AppError { message: "Timetable for this academic year already exists".into() });
                }
            }
        }

        let mut sql = QueryBuilder::<Postgres>::new("UPDATE school_timetables SET updated_at = now()");
        let mut touched = false;

        if let Some(v) = &dto.school_id {
            sql.push(", school_id = ").push_bind(v.to_hex()); touched = true;
        }
        if let Some(v) = &dto.academic_year_id {
            sql.push(", academic_year_id = ").push_bind(v.map(|o| o.to_hex())); touched = true;
        }
        if let Some(ref v) = dto.default_weekly_schedule {
            let j = serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]));
            sql.push(", default_weekly_schedule = ").push_bind(j); touched = true;
        }
        if let Some(ref v) = dto.overrides {
            let j = serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]));
            sql.push(", overrides = ").push_bind(j); touched = true;
        }
        if let Some(ref v) = dto.events {
            let j = serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]));
            sql.push(", events = ").push_bind(j); touched = true;
        }

        if !touched { return Err(AppError { message: "No valid fields to update".into() }); }

        sql.push(" WHERE id = ").push_bind(&obj_id_str);
        sql.build().execute(&self.pool).await.map_err(Self::db_error)?;

        self.find_one(Some(id), None).await
    }

    pub async fn delete_timetable(&self, id: &IdType) -> Result<SchoolTimetable, AppError> {
        let existing = self.find_one(Some(id), None).await?;
        sqlx::query("DELETE FROM school_timetables WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(existing)
    }

    pub async fn generate_default(
        &self,
        school_id: &IdType,
        academic_year: &IdType,
    ) -> Result<SchoolTimetable, AppError> {
        let school_obj = IdType::to_object_id(school_id)?;
        let year_obj = IdType::to_object_id(academic_year)?;

        let days = [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri];
        let weekly = days.iter().map(|&d| DailySchoolSchedule {
            day: d,
            is_school_day: true,
            school_start_time: "08:30".into(),
            school_end_time: "17:00".into(),
            study_start_time: "09:00".into(),
            study_end_time: "17:00".into(),
            breaks: vec![
                TimeBlock { start_time: "10:20".into(), end_time: "10:40".into(), description: Some("Morning break".into()), title: "Break".into() },
                TimeBlock { start_time: "15:20".into(), end_time: "15:40".into(), description: Some("Afternoon break".into()), title: "Break".into() },
            ],
            lunch: Some(TimeBlock { start_time: "13:00".into(), end_time: "14:00".into(), description: Some("Time for lunch".into()), title: "Lunch".into() }),
            activities: vec![],
            special_type: DaySpecialType::Normal,
        }).collect();

        let timetable = SchoolTimetable {
            id: None,
            school_id: school_obj,
            academic_year_id: Some(year_obj),
            default_weekly_schedule: weekly,
            overrides: Some(vec![]),
            events: Some(vec![]),
            created_at: None,
            updated_at: None,
        };

        self.create(timetable).await
    }
}

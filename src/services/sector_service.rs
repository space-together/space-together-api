use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        sector::{Sector, SectorPartial},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    services::cloudinary_service::CloudinaryService,
    utils::object_id::ObjectId,
};

pub struct SectorService {
    pub pool: PgPool,
}

impl SectorService {
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

    fn row_to_sector(row: PgRow) -> Result<Sector, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let cs: Option<i32> = row.try_get("curriculum_start").ok().flatten();
        let ce: Option<i32> = row.try_get("curriculum_end").ok().flatten();
        Ok(Sector {
            id: Some(Self::parse_oid(&id, "id")?),
            name: row.try_get("name").map_err(Self::db_error)?,
            logo: row.try_get("logo").ok().flatten(),
            logo_id: row.try_get("logo_id").ok().flatten(),
            username: row.try_get("username").map_err(Self::db_error)?,
            description: row.try_get("description").ok().flatten(),
            curriculum: cs.zip(ce),
            country: row.try_get("country").map_err(Self::db_error)?,
            r#type: row.try_get("type").map_err(Self::db_error)?,
            disable: row.try_get("disable").ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    fn select_sql() -> &'static str {
        "SELECT id, name, logo, logo_id, username, description, \
         curriculum_start, curriculum_end, country, type, disable, \
         created_at, updated_at FROM sectors WHERE 1=1"
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str) {
        sql.push(" AND (lower(name) LIKE ").push_bind(like)
            .push(" OR lower(username) LIKE ").push_bind(like)
            .push(" OR lower(country) LIKE ").push_bind(like)
            .push(" OR lower(type) LIKE ").push_bind(like)
            .push(" OR lower(coalesce(description,'')) LIKE ").push_bind(like)
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
            for id in &q.by_ids {
                sep.push_bind(id);
            }
            sep.push_unseparated(")");
        }

        if q.field.len() != q.value.len() {
            return Err(AppError { message: "Number of fields must match number of values".into() });
        }
        for (field, value) in q.field.iter().zip(q.value.iter()) {
            match field.as_str() {
                "id" | "_id" => { sql.push(" AND id = ").push_bind(value); }
                "name" => { sql.push(" AND lower(name) = lower(").push_bind(value).push(")"); }
                "username" => { sql.push(" AND lower(username) = lower(").push_bind(value).push(")"); }
                "country" => { sql.push(" AND lower(country) = lower(").push_bind(value).push(")"); }
                "type" => { sql.push(" AND lower(type) = lower(").push_bind(value).push(")"); }
                "disable" => {
                    let b = value.parse::<bool>().map_err(|_| AppError {
                        message: "Invalid disable filter value".into(),
                    })?;
                    sql.push(" AND disable = ").push_bind(b);
                }
                _ => return Err(AppError { message: format!("Unsupported filter field: {}", field) }),
            }
        }
        Ok(())
    }

    pub async fn create(&self, dto: Sector) -> Result<Sector, AppError> {
        if self.find_one_by_name(&dto.name).await?.is_some() {
            return Err(AppError { message: format!("Sector name already exists: {}", dto.name) });
        }
        if self.find_one_by_username(&dto.username).await?.is_some() {
            return Err(AppError { message: format!("Sector username already exists: {}", dto.username) });
        }

        let mut sector = dto;
        let id = Self::new_id();

        if let Some(logo) = sector.logo.clone() {
            let res = CloudinaryService::upload_to_cloudinary(&logo)
                .await
                .map_err(|e| AppError { message: e })?;
            sector.logo_id = Some(res.public_id);
            sector.logo = Some(res.secure_url);
        }

        let (cs, ce) = sector.curriculum.map_or((None, None), |(s, e)| (Some(s), Some(e)));
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO sectors (id, name, logo, logo_id, username, description,
               curriculum_start, curriculum_end, country, type, disable, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"#,
        )
        .bind(&id)
        .bind(&sector.name)
        .bind(&sector.logo)
        .bind(&sector.logo_id)
        .bind(&sector.username)
        .bind(&sector.description)
        .bind(cs)
        .bind(ce)
        .bind(&sector.country)
        .bind(&sector.r#type)
        .bind(sector.disable.unwrap_or(false))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    async fn find_one_by_name(&self, name: &str) -> Result<Option<Sector>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, logo, logo_id, username, description, curriculum_start, \
             curriculum_end, country, type, disable, created_at, updated_at \
             FROM sectors WHERE lower(name) = lower($1) LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        row.map(Self::row_to_sector).transpose()
    }

    async fn find_one_by_username(&self, username: &str) -> Result<Option<Sector>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, logo, logo_id, username, description, curriculum_start, \
             curriculum_end, country, type, disable, created_at, updated_at \
             FROM sectors WHERE lower(username) = lower($1) LIMIT 1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        row.map(Self::row_to_sector).transpose()
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
    ) -> Result<Sector, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_filters(&mut sql, query)?;
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_sector(r),
            None => Err(AppError { message: "Sector not found".into() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
    ) -> Result<Paginated<Sector>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter
            .as_ref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM sectors WHERE 1=1");
        Self::push_filters(&mut cq, query)?;
        if let Some(ref l) = like { Self::push_search(&mut cq, l); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_filters(&mut dq, query)?;
        if let Some(ref l) = like { Self::push_search(&mut dq, l); }
        dq.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit)
            .push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_sector).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated {
            data,
            total,
            total_pages: ((total as f64) / (limit as f64)).ceil() as i64,
            current_page: (skip / limit) + 1,
        })
    }

    pub async fn update(&self, id: &IdType, update: &SectorPartial) -> Result<Sector, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        if let Some(ref name) = update.name {
            if existing.name != *name && self.find_one_by_name(name).await?.is_some() {
                return Err(AppError { message: format!("Sector name already exists: {}", name) });
            }
        }
        if let Some(ref username) = update.username {
            if existing.username != *username && self.find_one_by_username(username).await?.is_some() {
                return Err(AppError { message: format!("Sector username already exists: {}", username) });
            }
        }

        let id_str = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE sectors SET updated_at = now()");
        let mut touched = false;

        if let Some(v) = &update.name { sql.push(", name = ").push_bind(v); touched = true; }
        if let Some(v) = &update.username { sql.push(", username = ").push_bind(v); touched = true; }
        if let Some(v) = &update.description { sql.push(", description = ").push_bind(v); touched = true; }
        if let Some(v) = &update.country { sql.push(", country = ").push_bind(v); touched = true; }
        if let Some(v) = &update.r#type { sql.push(", type = ").push_bind(v); touched = true; }
        if let Some(v) = update.disable { sql.push(", disable = ").push_bind(v); touched = true; }

        if let Some(curriculum_opt) = update.curriculum {
            match curriculum_opt {
                Some((s, e)) => {
                    sql.push(", curriculum_start = ").push_bind(s);
                    sql.push(", curriculum_end = ").push_bind(e);
                }
                None => {
                    sql.push(", curriculum_start = ").push_bind::<Option<i32>>(None);
                    sql.push(", curriculum_end = ").push_bind::<Option<i32>>(None);
                }
            }
            touched = true;
        }

        if let Some(logo_opt) = &update.logo {
            match logo_opt {
                Some(new_logo) if Some(new_logo.clone()) != existing.logo => {
                    if let Some(old_id) = &existing.logo_id {
                        CloudinaryService::delete_from_cloudinary(old_id).await.ok();
                    }
                    let res = CloudinaryService::upload_to_cloudinary(new_logo)
                        .await
                        .map_err(|e| AppError { message: e })?;
                    sql.push(", logo = ").push_bind(res.secure_url);
                    sql.push(", logo_id = ").push_bind(res.public_id);
                    touched = true;
                }
                None => {
                    if let Some(old_id) = &existing.logo_id {
                        CloudinaryService::delete_from_cloudinary(old_id).await.ok();
                    }
                    sql.push(", logo = ").push_bind::<Option<String>>(None);
                    sql.push(", logo_id = ").push_bind::<Option<String>>(None);
                    touched = true;
                }
                _ => {}
            }
        }

        if !touched {
            return Err(AppError { message: "No valid fields to update".into() });
        }

        sql.push(" WHERE id = ").push_bind(&id_str);
        sql.build().execute(&self.pool).await.map_err(Self::db_error)?;

        self.find_one(Some(id), None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Sector, AppError> {
        let sector = self.find_one(Some(id), None).await?;

        if let Some(ref logo_id) = sector.logo_id {
            CloudinaryService::delete_from_cloudinary(logo_id).await.ok();
        }

        sqlx::query("DELETE FROM sectors WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;

        Ok(sector)
    }

    pub async fn count(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
    ) -> Result<CountDoc, AppError> {
        let total = self.get_all(filter, Some(1), Some(0), query).await?.total;
        Ok(CountDoc { count: total as u64 })
    }

    pub async fn find_by_ids(&self, ids: Vec<IdType>) -> Result<Vec<Sector>, AppError> {
        let strings: Vec<String> = ids
            .into_iter()
            .filter_map(|id| IdType::to_object_id(&id).ok().map(|o| o.to_hex()))
            .collect();

        if strings.is_empty() {
            return Ok(vec![]);
        }

        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        sql.push(" AND id IN (");
        let mut sep = sql.separated(", ");
        for id in &strings { sep.push_bind(id); }
        sep.push_unseparated(")");

        let rows = sql.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        rows.into_iter().map(Self::row_to_sector).collect()
    }
}

use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        main_class::{MainClass, MainClassPartial, MainClassWithOthers},
        trade::Trade,
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    utils::object_id::ObjectId,
};

pub struct MainClassService {
    pub pool: PgPool,
}

impl MainClassService {
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

    pub fn row_to_main_class_pub(row: PgRow) -> Result<MainClass, AppError> {
        Self::row_to_main_class(row)
    }

    fn row_to_main_class(row: PgRow) -> Result<MainClass, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let trade_id_str: Option<String> = row.try_get("trade_id").ok().flatten();
        Ok(MainClass {
            id: Some(Self::parse_oid(&id, "id")?),
            name: row.try_get("name").map_err(Self::db_error)?,
            username: row.try_get("username").map_err(Self::db_error)?,
            trade_id: trade_id_str
                .as_deref()
                .map(|s| Self::parse_oid(s, "trade_id"))
                .transpose()?,
            level: row.try_get("level").ok().flatten(),
            description: row.try_get("description").ok().flatten(),
            disable: row.try_get("disable").ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    fn row_to_trade(row: &PgRow, prefix: &str) -> Result<Option<Trade>, AppError> {
        let id: Option<String> = row.try_get(format!("{}_id", prefix).as_str()).ok().flatten();
        let id = match id {
            Some(v) => v,
            None => return Ok(None),
        };
        Ok(Some(Trade {
            id: Some(Self::parse_oid(&id, "trade.id")?),
            sector_id: row
                .try_get::<Option<String>, _>(format!("{}_sector_id", prefix).as_str())
                .ok()
                .flatten()
                .as_deref()
                .map(|s| Self::parse_oid(s, "trade.sector_id"))
                .transpose()?,
            trade_id: row
                .try_get::<Option<String>, _>(format!("{}_trade_id", prefix).as_str())
                .ok()
                .flatten()
                .as_deref()
                .map(|s| Self::parse_oid(s, "trade.trade_id"))
                .transpose()?,
            name: row.try_get(format!("{}_name", prefix).as_str()).map_err(Self::db_error)?,
            username: row.try_get(format!("{}_username", prefix).as_str()).map_err(Self::db_error)?,
            description: row.try_get(format!("{}_description", prefix).as_str()).ok().flatten(),
            class_min: row.try_get(format!("{}_class_min", prefix).as_str()).unwrap_or(0),
            class_max: row.try_get(format!("{}_class_max", prefix).as_str()).unwrap_or(0),
            r#type: row
                .try_get::<String, _>(format!("{}_type", prefix).as_str())
                .unwrap_or_else(|_| "SENIOR".to_string())
                .into(),
            disable: row.try_get(format!("{}_disable", prefix).as_str()).ok(),
            created_at: row.try_get(format!("{}_created_at", prefix).as_str()).ok(),
            updated_at: row.try_get(format!("{}_updated_at", prefix).as_str()).ok(),
        }))
    }

    fn select_sql() -> &'static str {
        "SELECT id, name, username, trade_id, level, description, disable, \
         created_at, updated_at FROM main_classes WHERE 1=1"
    }

    fn with_relations_sql() -> &'static str {
        r#"SELECT mc.id, mc.name, mc.username, mc.trade_id, mc.level, mc.description,
                  mc.disable, mc.created_at, mc.updated_at,
                  t.id AS t_id, t.sector_id AS t_sector_id, t.trade_id AS t_trade_id,
                  t.name AS t_name, t.username AS t_username, t.description AS t_description,
                  t.class_min AS t_class_min, t.class_max AS t_class_max,
                  t.type AS t_type, t.disable AS t_disable,
                  t.created_at AS t_created_at, t.updated_at AS t_updated_at
           FROM main_classes mc
           LEFT JOIN trades t ON t.id = mc.trade_id
           WHERE 1=1"#
    }

    fn row_to_main_class_with_others(row: PgRow) -> Result<MainClassWithOthers, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        let trade_id_str: Option<String> = row.try_get("trade_id").ok().flatten();
        let main_class = MainClass {
            id: Some(Self::parse_oid(&id, "id")?),
            name: row.try_get("name").map_err(Self::db_error)?,
            username: row.try_get("username").map_err(Self::db_error)?,
            trade_id: trade_id_str
                .as_deref()
                .map(|s| Self::parse_oid(s, "trade_id"))
                .transpose()?,
            level: row.try_get("level").ok().flatten(),
            description: row.try_get("description").ok().flatten(),
            disable: row.try_get("disable").ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        };
        let trade = Self::row_to_trade(&row, "t")?;
        Ok(MainClassWithOthers { main_class, trade })
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str) {
        sql.push(" AND (lower(mc.name) LIKE ").push_bind(like)
            .push(" OR lower(mc.username) LIKE ").push_bind(like)
            .push(" OR lower(coalesce(mc.description,'')) LIKE ").push_bind(like)
            .push(" OR lower(mc.id) LIKE ").push_bind(like)
            .push(")");
    }

    fn push_search_simple<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str) {
        sql.push(" AND (lower(name) LIKE ").push_bind(like)
            .push(" OR lower(username) LIKE ").push_bind(like)
            .push(" OR lower(coalesce(description,'')) LIKE ").push_bind(like)
            .push(" OR lower(id) LIKE ").push_bind(like)
            .push(")");
    }

    fn push_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RequestQuery>,
        aliased: bool,
    ) -> Result<(), AppError> {
        let Some(q) = query else { return Ok(()) };
        let prefix = if aliased { "mc." } else { "" };

        if !q.by_ids.is_empty() {
            sql.push(format!(" AND {}id IN (", prefix));
            let mut sep = sql.separated(", ");
            for id in &q.by_ids { sep.push_bind(id); }
            sep.push_unseparated(")");
        }

        if q.field.len() != q.value.len() {
            return Err(AppError { message: "Number of fields must match number of values".into() });
        }
        for (field, value) in q.field.iter().zip(q.value.iter()) {
            match field.as_str() {
                "id" | "_id" => { sql.push(format!(" AND {}id = ", prefix)).push_bind(value); }
                "trade_id" => { sql.push(format!(" AND {}trade_id = ", prefix)).push_bind(value); }
                "name" => { sql.push(format!(" AND lower({}name) = lower(", prefix)).push_bind(value).push(")"); }
                "username" => { sql.push(format!(" AND lower({}username) = lower(", prefix)).push_bind(value).push(")"); }
                "level" => {
                    let n = value.parse::<i32>().map_err(|_| AppError { message: "Invalid level".into() })?;
                    sql.push(format!(" AND {}level = ", prefix)).push_bind(n);
                }
                "disable" => {
                    let b = value.parse::<bool>().map_err(|_| AppError { message: "Invalid disable".into() })?;
                    sql.push(format!(" AND {}disable = ", prefix)).push_bind(b);
                }
                _ => return Err(AppError { message: format!("Unsupported filter field: {}", field) }),
            }
        }
        Ok(())
    }

    pub async fn create(&self, dto: MainClass) -> Result<MainClass, AppError> {
        if self.find_one_by_name(&dto.name).await?.is_some() {
            return Err(AppError { message: format!("MainClass name already exists: {}", dto.name) });
        }
        if self.find_one_by_username(&dto.username).await?.is_some() {
            return Err(AppError { message: format!("MainClass username already exists: {}", dto.username) });
        }

        let id = Self::new_id();
        let trade_id = dto.trade_id.as_ref().map(|o| o.to_hex());
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO main_classes (id, name, username, trade_id, level, description, disable, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
        )
        .bind(&id)
        .bind(&dto.name)
        .bind(&dto.username)
        .bind(&trade_id)
        .bind(dto.level)
        .bind(&dto.description)
        .bind(dto.disable.unwrap_or(false))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        self.find_one(Some(&IdType::from_string(id)), None).await
    }

    async fn find_one_by_name(&self, name: &str) -> Result<Option<MainClass>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, username, trade_id, level, description, disable, created_at, updated_at \
             FROM main_classes WHERE lower(name) = lower($1) LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        row.map(Self::row_to_main_class).transpose()
    }

    async fn find_one_by_username(&self, username: &str) -> Result<Option<MainClass>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, username, trade_id, level, description, disable, created_at, updated_at \
             FROM main_classes WHERE lower(username) = lower($1) LIMIT 1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        row.map(Self::row_to_main_class).transpose()
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
    ) -> Result<MainClass, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_filters(&mut sql, query, false)?;
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_main_class(r),
            None => Err(AppError { message: "MainClass not found".into() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
    ) -> Result<Paginated<MainClass>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter
            .as_ref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM main_classes WHERE 1=1");
        Self::push_filters(&mut cq, query, false)?;
        if let Some(ref l) = like { Self::push_search_simple(&mut cq, l); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_filters(&mut dq, query, false)?;
        if let Some(ref l) = like { Self::push_search_simple(&mut dq, l); }
        dq.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit)
            .push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_main_class).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
    ) -> Result<Paginated<MainClassWithOthers>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter
            .as_ref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM main_classes mc LEFT JOIN trades t ON t.id = mc.trade_id WHERE 1=1",
        );
        Self::push_filters(&mut cq, query, true)?;
        if let Some(ref l) = like { Self::push_search(&mut cq, l); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::with_relations_sql());
        Self::push_filters(&mut dq, query, true)?;
        if let Some(ref l) = like { Self::push_search(&mut dq, l); }
        dq.push(" ORDER BY mc.updated_at DESC LIMIT ").push_bind(limit)
            .push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_main_class_with_others).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
    ) -> Result<MainClassWithOthers, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::with_relations_sql());
        if let Some(id) = id {
            sql.push(" AND mc.id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_filters(&mut sql, query, true)?;
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_main_class_with_others(r),
            None => Err(AppError { message: "MainClass not found".into() }),
        }
    }

    pub async fn update(
        &self,
        id: &IdType,
        update: &MainClassPartial,
    ) -> Result<MainClass, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        if let Some(ref name) = update.name {
            if existing.name != *name && self.find_one_by_name(name).await?.is_some() {
                return Err(AppError { message: format!("MainClass name already exists: {}", name) });
            }
        }
        if let Some(ref username) = update.username {
            if existing.username != *username && self.find_one_by_username(username).await?.is_some() {
                return Err(AppError { message: format!("MainClass username already exists: {}", username) });
            }
        }

        let id_str = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE main_classes SET updated_at = now()");
        let mut touched = false;

        if let Some(v) = &update.name { sql.push(", name = ").push_bind(v); touched = true; }
        if let Some(v) = &update.username { sql.push(", username = ").push_bind(v); touched = true; }
        if let Some(v) = &update.description { sql.push(", description = ").push_bind(v); touched = true; }
        if let Some(v) = update.level { sql.push(", level = ").push_bind(v); touched = true; }
        if let Some(v) = update.disable { sql.push(", disable = ").push_bind(v); touched = true; }
        if let Some(v) = update.trade_id {
            sql.push(", trade_id = ").push_bind(v.map(|o| o.to_hex()));
            touched = true;
        }

        if !touched {
            return Err(AppError { message: "No valid fields to update".into() });
        }

        sql.push(" WHERE id = ").push_bind(&id_str);
        sql.build().execute(&self.pool).await.map_err(Self::db_error)?;

        self.find_one(Some(id), None).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<MainClass, AppError> {
        let mc = self.find_one(Some(id), None).await?;
        sqlx::query("DELETE FROM main_classes WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(mc)
    }

    pub async fn count(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
    ) -> Result<CountDoc, AppError> {
        let total = self.get_all(filter, Some(1), Some(0), query).await?.total;
        Ok(CountDoc { count: total as u64 })
    }

    pub async fn create_many(&self, classes: Vec<MainClass>) -> Result<Vec<MainClass>, AppError> {
        let mut result = Vec::with_capacity(classes.len());
        for cls in classes {
            let mc = self.create(cls).await?;
            result.push(mc);
        }
        Ok(result)
    }
}

use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use crate::{
    domain::{
        common_details::Paginated,
        main_class::MainClass,
        sector::Sector,
        trade::{Trade, TradePartial, TradeWithOthers, TradeWithParent},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType, mongo_model::CountDoc},
    services::main_class_service::MainClassService,
    utils::object_id::ObjectId,
};

pub struct TradeService {
    pub pool: PgPool,
}

impl TradeService {
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

    fn row_to_trade(row: &PgRow) -> Result<Trade, AppError> {
        let id: String = row.try_get("id").map_err(Self::db_error)?;
        Ok(Trade {
            id: Some(Self::parse_oid(&id, "id")?),
            sector_id: row
                .try_get::<Option<String>, _>("sector_id")
                .ok()
                .flatten()
                .as_deref()
                .map(|s| Self::parse_oid(s, "sector_id"))
                .transpose()?,
            trade_id: row
                .try_get::<Option<String>, _>("trade_id")
                .ok()
                .flatten()
                .as_deref()
                .map(|s| Self::parse_oid(s, "trade_id"))
                .transpose()?,
            name: row.try_get("name").map_err(Self::db_error)?,
            username: row.try_get("username").map_err(Self::db_error)?,
            description: row.try_get("description").ok().flatten(),
            class_min: row.try_get("class_min").unwrap_or(0),
            class_max: row.try_get("class_max").unwrap_or(0),
            r#type: row
                .try_get::<String, _>("type")
                .unwrap_or_else(|_| "SENIOR".to_string())
                .into(),
            disable: row.try_get("disable").ok(),
            created_at: row.try_get("created_at").ok(),
            updated_at: row.try_get("updated_at").ok(),
        })
    }

    fn row_to_sector_prefixed(row: &PgRow, prefix: &str) -> Result<Option<Sector>, AppError> {
        let id: Option<String> = row
            .try_get::<Option<String>, _>(format!("{}_id", prefix).as_str())
            .ok()
            .flatten();
        let id = match id {
            Some(v) => v,
            None => return Ok(None),
        };
        Ok(Some(Sector {
            id: Some(Self::parse_oid(&id, "sector.id")?),
            name: row.try_get(format!("{}_name", prefix).as_str()).map_err(Self::db_error)?,
            logo: row.try_get(format!("{}_logo", prefix).as_str()).ok().flatten(),
            logo_id: row.try_get(format!("{}_logo_id", prefix).as_str()).ok().flatten(),
            username: row.try_get(format!("{}_username", prefix).as_str()).map_err(Self::db_error)?,
            description: row.try_get(format!("{}_description", prefix).as_str()).ok().flatten(),
            curriculum: {
                let cs: Option<i32> = row.try_get(format!("{}_curriculum_start", prefix).as_str()).ok().flatten();
                let ce: Option<i32> = row.try_get(format!("{}_curriculum_end", prefix).as_str()).ok().flatten();
                cs.zip(ce)
            },
            country: row.try_get(format!("{}_country", prefix).as_str()).map_err(Self::db_error)?,
            r#type: row.try_get(format!("{}_type", prefix).as_str()).map_err(Self::db_error)?,
            disable: row.try_get(format!("{}_disable", prefix).as_str()).ok(),
            created_at: row.try_get(format!("{}_created_at", prefix).as_str()).ok(),
            updated_at: row.try_get(format!("{}_updated_at", prefix).as_str()).ok(),
        }))
    }

    fn select_sql() -> &'static str {
        "SELECT id, sector_id, trade_id, name, username, description, \
         class_min, class_max, type, disable, created_at, updated_at \
         FROM trades WHERE 1=1"
    }

    fn with_relations_sql() -> &'static str {
        r#"SELECT t.id, t.sector_id, t.trade_id, t.name, t.username, t.description,
                  t.class_min, t.class_max, t.type, t.disable, t.created_at, t.updated_at,
                  s.id AS s_id, s.name AS s_name, s.logo AS s_logo, s.logo_id AS s_logo_id,
                  s.username AS s_username, s.description AS s_description,
                  s.curriculum_start AS s_curriculum_start, s.curriculum_end AS s_curriculum_end,
                  s.country AS s_country, s.type AS s_type, s.disable AS s_disable,
                  s.created_at AS s_created_at, s.updated_at AS s_updated_at,
                  pt.id AS pt_id, pt.sector_id AS pt_sector_id, pt.trade_id AS pt_trade_id,
                  pt.name AS pt_name, pt.username AS pt_username, pt.description AS pt_description,
                  pt.class_min AS pt_class_min, pt.class_max AS pt_class_max,
                  pt.type AS pt_type, pt.disable AS pt_disable,
                  pt.created_at AS pt_created_at, pt.updated_at AS pt_updated_at,
                  ps.id AS ps_id, ps.name AS ps_name, ps.logo AS ps_logo, ps.logo_id AS ps_logo_id,
                  ps.username AS ps_username, ps.description AS ps_description,
                  ps.curriculum_start AS ps_curriculum_start, ps.curriculum_end AS ps_curriculum_end,
                  ps.country AS ps_country, ps.type AS ps_type, ps.disable AS ps_disable,
                  ps.created_at AS ps_created_at, ps.updated_at AS ps_updated_at
           FROM trades t
           LEFT JOIN sectors s ON s.id = t.sector_id
           LEFT JOIN trades pt ON pt.id = t.trade_id
           LEFT JOIN sectors ps ON ps.id = pt.sector_id
           WHERE 1=1"#
    }

    fn row_to_trade_with_others(row: PgRow) -> Result<TradeWithOthers, AppError> {
        let trade = Self::row_to_trade(&row)?;
        let sector = Self::row_to_sector_prefixed(&row, "s")?;

        let parent_trade = {
            let pt_id: Option<String> = row.try_get("pt_id").ok().flatten();
            if pt_id.is_none() {
                None
            } else {
                let pt = Trade {
                    id: pt_id.as_deref().map(|s| Self::parse_oid(s, "pt.id")).transpose()?,
                    sector_id: row
                        .try_get::<Option<String>, _>("pt_sector_id")
                        .ok()
                        .flatten()
                        .as_deref()
                        .map(|s| Self::parse_oid(s, "pt.sector_id"))
                        .transpose()?,
                    trade_id: row
                        .try_get::<Option<String>, _>("pt_trade_id")
                        .ok()
                        .flatten()
                        .as_deref()
                        .map(|s| Self::parse_oid(s, "pt.trade_id"))
                        .transpose()?,
                    name: row.try_get("pt_name").map_err(Self::db_error)?,
                    username: row.try_get("pt_username").map_err(Self::db_error)?,
                    description: row.try_get("pt_description").ok().flatten(),
                    class_min: row.try_get("pt_class_min").unwrap_or(0),
                    class_max: row.try_get("pt_class_max").unwrap_or(0),
                    r#type: row.try_get::<String, _>("pt_type").unwrap_or_else(|_| "SENIOR".to_string()).into(),
                    disable: row.try_get("pt_disable").ok(),
                    created_at: row.try_get("pt_created_at").ok(),
                    updated_at: row.try_get("pt_updated_at").ok(),
                };
                let pt_sector = Self::row_to_sector_prefixed(&row, "ps")?;
                Some(Box::new(TradeWithParent { trade: pt, sector: pt_sector }))
            }
        };

        Ok(TradeWithOthers { trade, sector, parent_trade })
    }

    fn push_search<'a>(sql: &mut QueryBuilder<'a, Postgres>, like: &'a str, aliased: bool) {
        let t = if aliased { "t." } else { "" };
        sql.push(format!(" AND (lower({t}name) LIKE ")).push_bind(like)
            .push(format!(" OR lower({t}username) LIKE ")).push_bind(like)
            .push(format!(" OR lower(coalesce({t}description,'')) LIKE ")).push_bind(like)
            .push(format!(" OR lower({t}type) LIKE ")).push_bind(like)
            .push(format!(" OR lower({t}id) LIKE ")).push_bind(like)
            .push(")");
    }

    fn push_filters<'a>(
        sql: &mut QueryBuilder<'a, Postgres>,
        query: Option<&'a RequestQuery>,
        aliased: bool,
    ) -> Result<(), AppError> {
        let Some(q) = query else { return Ok(()) };
        let t = if aliased { "t." } else { "" };

        if !q.by_ids.is_empty() {
            sql.push(format!(" AND {t}id IN ("));
            let mut sep = sql.separated(", ");
            for id in &q.by_ids { sep.push_bind(id); }
            sep.push_unseparated(")");
        }

        if q.field.len() != q.value.len() {
            return Err(AppError { message: "Number of fields must match number of values".into() });
        }
        for (field, value) in q.field.iter().zip(q.value.iter()) {
            match field.as_str() {
                "id" | "_id" => { sql.push(format!(" AND {t}id = ")).push_bind(value); }
                "sector_id" => { sql.push(format!(" AND {t}sector_id = ")).push_bind(value); }
                "trade_id" => { sql.push(format!(" AND {t}trade_id = ")).push_bind(value); }
                "name" => { sql.push(format!(" AND lower({t}name) = lower(")).push_bind(value).push(")"); }
                "username" => { sql.push(format!(" AND lower({t}username) = lower(")).push_bind(value).push(")"); }
                "type" => { sql.push(format!(" AND lower({t}type) = lower(")).push_bind(value).push(")"); }
                "disable" => {
                    let b = value.parse::<bool>().map_err(|_| AppError { message: "Invalid disable".into() })?;
                    sql.push(format!(" AND {t}disable = ")).push_bind(b);
                }
                _ => return Err(AppError { message: format!("Unsupported filter field: {}", field) }),
            }
        }
        Ok(())
    }

    pub async fn create(&self, dto: Trade) -> Result<Trade, AppError> {
        if self.find_one_by_name(&dto.name).await?.is_some() {
            return Err(AppError { message: format!("Trade name already exists: {}", dto.name) });
        }
        if self.find_one_by_username(&dto.username).await?.is_some() {
            return Err(AppError { message: format!("Trade username already exists: {}", dto.username) });
        }

        let id = Self::new_id();
        let sector_id = dto.sector_id.as_ref().map(|o| o.to_hex());
        let parent_trade_id = dto.trade_id.as_ref().map(|o| o.to_hex());
        let trade_type: String = dto.r#type.clone().into();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO trades (id, sector_id, trade_id, name, username, description,
               class_min, class_max, type, disable, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"#,
        )
        .bind(&id)
        .bind(&sector_id)
        .bind(&parent_trade_id)
        .bind(&dto.name)
        .bind(&dto.username)
        .bind(&dto.description)
        .bind(dto.class_min)
        .bind(dto.class_max)
        .bind(&trade_type)
        .bind(dto.disable.unwrap_or(false))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(Self::db_error)?;

        let trade = self.find_one(Some(&IdType::from_string(id.clone())), None).await?;

        // Auto-create main classes for the trade range
        let mut min = dto.class_min;
        let mut max = dto.class_max;
        if min > max { std::mem::swap(&mut min, &mut max); }

        if max >= min {
            let trade_type_name = dto.r#type.to_string();
            let trade_oid = Self::parse_oid(&id, "trade.id")?;
            let mc_service = MainClassService::new(&self.pool);
            let mut to_create = Vec::new();

            for level in min..=max {
                let name = format!("{} {} {}", trade_type_name, level, dto.name);
                let username = format!(
                    "{}_{}_{}",
                    trade_type_name.to_lowercase(),
                    level,
                    dto.username.replace(' ', "_").to_lowercase()
                );
                to_create.push(MainClass {
                    id: None,
                    name,
                    username,
                    trade_id: Some(trade_oid.clone()),
                    level: Some(level),
                    description: Some(format!("Auto-created for trade {}", dto.name)),
                    disable: Some(false),
                    created_at: Some(now),
                    updated_at: Some(now),
                });
            }

            if !to_create.is_empty() {
                let _ = mc_service.create_many(to_create).await;
            }
        }

        Ok(trade)
    }

    async fn find_one_by_name(&self, name: &str) -> Result<Option<Trade>, AppError> {
        let row = sqlx::query(
            "SELECT id, sector_id, trade_id, name, username, description, class_min, class_max, \
             type, disable, created_at, updated_at FROM trades WHERE lower(name) = lower($1) LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        row.as_ref().map(|r| Self::row_to_trade(r)).transpose()
    }

    async fn find_one_by_username(&self, username: &str) -> Result<Option<Trade>, AppError> {
        let row = sqlx::query(
            "SELECT id, sector_id, trade_id, name, username, description, class_min, class_max, \
             type, disable, created_at, updated_at FROM trades WHERE lower(username) = lower($1) LIMIT 1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::db_error)?;
        row.as_ref().map(|r| Self::row_to_trade(r)).transpose()
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
    ) -> Result<Trade, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        if let Some(id) = id {
            sql.push(" AND id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_filters(&mut sql, query, false)?;
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_trade(&r),
            None => Err(AppError { message: "Trade not found".into() }),
        }
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        query: Option<&RequestQuery>,
    ) -> Result<TradeWithOthers, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(Self::with_relations_sql());
        if let Some(id) = id {
            sql.push(" AND t.id = ").push_bind(Self::id_to_string(id)?);
        }
        Self::push_filters(&mut sql, query, true)?;
        sql.push(" LIMIT 1");

        let row = sql.build().fetch_optional(&self.pool).await.map_err(Self::db_error)?;
        match row {
            Some(r) => Self::row_to_trade_with_others(r),
            None => Err(AppError { message: "Trade not found".into() }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
    ) -> Result<Paginated<Trade>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter
            .as_ref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new("SELECT count(*) FROM trades WHERE 1=1");
        Self::push_filters(&mut cq, query, false)?;
        if let Some(ref l) = like { Self::push_search(&mut cq, l, false); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::select_sql());
        Self::push_filters(&mut dq, query, false)?;
        if let Some(ref l) = like { Self::push_search(&mut dq, l, false); }
        dq.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit)
            .push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.iter().map(Self::row_to_trade).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        query: Option<&RequestQuery>,
    ) -> Result<Paginated<TradeWithOthers>, AppError> {
        let limit = limit.unwrap_or(20).max(1);
        let skip = skip.unwrap_or(0).max(0);
        let like = filter
            .as_ref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("%{}%", v.to_lowercase()));

        let mut cq = QueryBuilder::<Postgres>::new(
            "SELECT count(*) FROM trades t \
             LEFT JOIN sectors s ON s.id = t.sector_id \
             LEFT JOIN trades pt ON pt.id = t.trade_id \
             LEFT JOIN sectors ps ON ps.id = pt.sector_id \
             WHERE 1=1",
        );
        Self::push_filters(&mut cq, query, true)?;
        if let Some(ref l) = like { Self::push_search(&mut cq, l, true); }
        let total: i64 = cq.build_query_scalar().fetch_one(&self.pool).await.map_err(Self::db_error)?;

        let mut dq = QueryBuilder::<Postgres>::new(Self::with_relations_sql());
        Self::push_filters(&mut dq, query, true)?;
        if let Some(ref l) = like { Self::push_search(&mut dq, l, true); }
        dq.push(" ORDER BY t.updated_at DESC LIMIT ").push_bind(limit)
            .push(" OFFSET ").push_bind(skip);

        let rows = dq.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        let data = rows.into_iter().map(Self::row_to_trade_with_others).collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated { data, total, total_pages: ((total as f64) / (limit as f64)).ceil() as i64, current_page: (skip / limit) + 1 })
    }

    pub async fn update(&self, id: &IdType, update: &TradePartial) -> Result<Trade, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        if let Some(ref name) = update.name {
            if existing.name != *name && self.find_one_by_name(name).await?.is_some() {
                return Err(AppError { message: format!("Trade name already exists: {}", name) });
            }
        }
        if let Some(ref username) = update.username {
            if existing.username != *username && self.find_one_by_username(username).await?.is_some() {
                return Err(AppError { message: format!("Trade username already exists: {}", username) });
            }
        }

        let id_str = Self::id_to_string(id)?;
        let mut sql = QueryBuilder::<Postgres>::new("UPDATE trades SET updated_at = now()");
        let mut touched = false;

        if let Some(v) = &update.name { sql.push(", name = ").push_bind(v); touched = true; }
        if let Some(v) = &update.username { sql.push(", username = ").push_bind(v); touched = true; }
        if let Some(v) = &update.description { sql.push(", description = ").push_bind(v); touched = true; }
        if let Some(v) = update.class_min { sql.push(", class_min = ").push_bind(v); touched = true; }
        if let Some(v) = update.class_max { sql.push(", class_max = ").push_bind(v); touched = true; }
        if let Some(v) = update.disable { sql.push(", disable = ").push_bind(v); touched = true; }
        if let Some(ref t) = update.r#type {
            sql.push(", type = ").push_bind::<String>(t.clone().into());
            touched = true;
        }
        if let Some(v) = update.sector_id {
            sql.push(", sector_id = ").push_bind(v.map(|o| o.to_hex()));
            touched = true;
        }
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

    pub async fn delete(&self, id: &IdType) -> Result<Trade, AppError> {
        let trade = self.find_one(Some(id), None).await?;
        sqlx::query("DELETE FROM trades WHERE id = $1")
            .bind(Self::id_to_string(id)?)
            .execute(&self.pool)
            .await
            .map_err(Self::db_error)?;
        Ok(trade)
    }

    pub async fn count(
        &self,
        filter: Option<String>,
        query: Option<&RequestQuery>,
    ) -> Result<CountDoc, AppError> {
        let total = self.get_all(filter, Some(1), Some(0), query).await?.total;
        Ok(CountDoc { count: total as u64 })
    }

    pub async fn find_by_ids(&self, ids: Vec<IdType>) -> Result<Vec<Trade>, AppError> {
        let strings: Vec<String> = ids
            .into_iter()
            .filter_map(|id| IdType::to_object_id(&id).ok().map(|o| o.to_hex()))
            .collect();
        if strings.is_empty() { return Ok(vec![]); }

        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        sql.push(" AND id IN (");
        let mut sep = sql.separated(", ");
        for id in &strings { sep.push_bind(id); }
        sep.push_unseparated(")");

        let rows = sql.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        rows.iter().map(Self::row_to_trade).collect()
    }

    pub async fn get_trades_by_sector_ids(
        &self,
        sector_ids: &[ObjectId],
    ) -> Result<Vec<Trade>, AppError> {
        if sector_ids.is_empty() { return Ok(vec![]); }
        let strings: Vec<String> = sector_ids.iter().map(|o| o.to_hex()).collect();

        let mut sql = QueryBuilder::<Postgres>::new(Self::select_sql());
        sql.push(" AND sector_id IN (");
        let mut sep = sql.separated(", ");
        for id in &strings { sep.push_bind(id); }
        sep.push_unseparated(")");

        let rows = sql.build().fetch_all(&self.pool).await.map_err(Self::db_error)?;
        rows.iter().map(Self::row_to_trade).collect()
    }

    pub async fn get_trades_by_ids(
        &self,
        trade_ids: &[ObjectId],
    ) -> Result<Vec<Trade>, AppError> {
        self.find_by_ids(trade_ids.iter().map(|o| IdType::from_object_id(o.clone())).collect()).await
    }
}

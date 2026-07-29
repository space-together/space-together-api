use serde::Deserialize;
use sqlx::{PgPool, QueryBuilder};

use crate::{domain::location::LocationOption, errors::AppError, utils::object_id::ObjectId};

#[derive(Debug, Deserialize)]
struct RawLocations {
    provinces: Vec<RawProvince>,
}

#[derive(Debug, Deserialize)]
struct RawProvince {
    name: String,
    districts: Vec<RawDistrict>,
}

#[derive(Debug, Deserialize)]
struct RawDistrict {
    name: String,
    sectors: Vec<RawSector>,
}

#[derive(Debug, Deserialize)]
struct RawSector {
    name: String,
    cells: Vec<RawCell>,
}

#[derive(Debug, Deserialize)]
struct RawCell {
    name: String,
    villages: Vec<RawVillage>,
}

#[derive(Debug, Deserialize)]
struct RawVillage {
    name: String,
}

struct LocationSeedRow {
    id: String,
    province: String,
    district: String,
    sector: String,
    cell: String,
    village: String,
    province_order: i32,
    district_order: i32,
    sector_order: i32,
    cell_order: i32,
    village_order: i32,
}

pub struct LocationService {
    pool: PgPool,
}

impl LocationService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    fn db_error(err: sqlx::Error) -> AppError {
        AppError {
            message: format!("PostgreSQL Error: {}", err),
        }
    }

    async fn ensure_seeded(&self) -> Result<(), AppError> {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM locations")
            .fetch_one(&self.pool)
            .await
            .map_err(Self::db_error)?;

        if count > 0 {
            return Ok(());
        }

        let raw: RawLocations = serde_json::from_str(include_str!("../../data/locations.json"))
            .map_err(|err| AppError {
                message: format!("Failed to parse locations data: {}", err),
            })?;

        let mut rows = Vec::new();
        for (province_index, province) in raw.provinces.into_iter().enumerate() {
            for (district_index, district) in province.districts.into_iter().enumerate() {
                for (sector_index, sector) in district.sectors.into_iter().enumerate() {
                    for (cell_index, cell) in sector.cells.into_iter().enumerate() {
                        for (village_index, village) in cell.villages.into_iter().enumerate() {
                            rows.push(LocationSeedRow {
                                id: ObjectId::new().to_hex(),
                                province: province.name.clone(),
                                district: district.name.clone(),
                                sector: sector.name.clone(),
                                cell: cell.name.clone(),
                                village: village.name,
                                province_order: province_index as i32,
                                district_order: district_index as i32,
                                sector_order: sector_index as i32,
                                cell_order: cell_index as i32,
                                village_order: village_index as i32,
                            });
                        }
                    }
                }
            }
        }

        for chunk in rows.chunks(1000) {
            let mut builder = QueryBuilder::new(
                "INSERT INTO locations (
                    id, country, province, district, sector, cell, village,
                    province_order, district_order, sector_order, cell_order, village_order
                ) ",
            );

            builder.push_values(chunk, |mut row, item| {
                row.push_bind(&item.id)
                    .push_bind("Rwanda")
                    .push_bind(&item.province)
                    .push_bind(&item.district)
                    .push_bind(&item.sector)
                    .push_bind(&item.cell)
                    .push_bind(&item.village)
                    .push_bind(item.province_order)
                    .push_bind(item.district_order)
                    .push_bind(item.sector_order)
                    .push_bind(item.cell_order)
                    .push_bind(item.village_order);
            });

            builder.push(" ON CONFLICT DO NOTHING");
            builder
                .build()
                .execute(&self.pool)
                .await
                .map_err(Self::db_error)?;
        }

        Ok(())
    }

    fn country(country: Option<&str>) -> &str {
        country
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Rwanda")
    }

    pub async fn provinces(&self, country: Option<&str>) -> Result<Vec<LocationOption>, AppError> {
        self.ensure_seeded().await?;
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT province
             FROM locations
             WHERE lower(country) = lower($1)
             GROUP BY province, province_order
             ORDER BY province_order, province",
        )
        .bind(Self::country(country))
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(rows
            .into_iter()
            .map(|(name,)| LocationOption { name })
            .collect())
    }

    pub async fn districts(
        &self,
        country: Option<&str>,
        province: &str,
    ) -> Result<Vec<LocationOption>, AppError> {
        self.ensure_seeded().await?;
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT district
             FROM locations
             WHERE lower(country) = lower($1) AND lower(province) = lower($2)
             GROUP BY district, district_order
             ORDER BY district_order, district",
        )
        .bind(Self::country(country))
        .bind(province)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(rows
            .into_iter()
            .map(|(name,)| LocationOption { name })
            .collect())
    }

    pub async fn sectors(
        &self,
        country: Option<&str>,
        province: &str,
        district: &str,
    ) -> Result<Vec<LocationOption>, AppError> {
        self.ensure_seeded().await?;
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT sector
             FROM locations
             WHERE lower(country) = lower($1)
               AND lower(province) = lower($2)
               AND lower(district) = lower($3)
             GROUP BY sector, sector_order
             ORDER BY sector_order, sector",
        )
        .bind(Self::country(country))
        .bind(province)
        .bind(district)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(rows
            .into_iter()
            .map(|(name,)| LocationOption { name })
            .collect())
    }

    pub async fn cells(
        &self,
        country: Option<&str>,
        province: &str,
        district: &str,
        sector: &str,
    ) -> Result<Vec<LocationOption>, AppError> {
        self.ensure_seeded().await?;
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT cell
             FROM locations
             WHERE lower(country) = lower($1)
               AND lower(province) = lower($2)
               AND lower(district) = lower($3)
               AND lower(sector) = lower($4)
             GROUP BY cell, cell_order
             ORDER BY cell_order, cell",
        )
        .bind(Self::country(country))
        .bind(province)
        .bind(district)
        .bind(sector)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(rows
            .into_iter()
            .map(|(name,)| LocationOption { name })
            .collect())
    }

    pub async fn villages(
        &self,
        country: Option<&str>,
        province: &str,
        district: &str,
        sector: &str,
        cell: &str,
    ) -> Result<Vec<LocationOption>, AppError> {
        self.ensure_seeded().await?;
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT village
             FROM locations
             WHERE lower(country) = lower($1)
               AND lower(province) = lower($2)
               AND lower(district) = lower($3)
               AND lower(sector) = lower($4)
               AND lower(cell) = lower($5)
             GROUP BY village, village_order
             ORDER BY village_order, village",
        )
        .bind(Self::country(country))
        .bind(province)
        .bind(district)
        .bind(sector)
        .bind(cell)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::db_error)?;

        Ok(rows
            .into_iter()
            .map(|(name,)| LocationOption { name })
            .collect())
    }
}

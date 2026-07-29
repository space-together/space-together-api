use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRow {
    pub id: String,
    pub country: String,
    pub province: String,
    pub district: String,
    pub sector: String,
    pub cell: String,
    pub village: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationOption {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ProvinceQuery {
    pub country: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DistrictQuery {
    pub country: Option<String>,
    pub province: String,
}

#[derive(Debug, Deserialize)]
pub struct SectorQuery {
    pub country: Option<String>,
    pub province: String,
    pub district: String,
}

#[derive(Debug, Deserialize)]
pub struct CellQuery {
    pub country: Option<String>,
    pub province: String,
    pub district: String,
    pub sector: String,
}

#[derive(Debug, Deserialize)]
pub struct VillageQuery {
    pub country: Option<String>,
    pub province: String,
    pub district: String,
    pub sector: String,
    pub cell: String,
}

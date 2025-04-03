use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressModel {
    pub country: String,
    pub province: String,
    pub district: String,
    pub sector: Option<String>,
    pub cell: Option<String>,
    pub village: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub google_map_uri: Option<String>,
}

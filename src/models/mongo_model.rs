use serde::{Deserialize, Serialize};

// count doc
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountDoc {
    pub count: u64,
}

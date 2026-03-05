use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

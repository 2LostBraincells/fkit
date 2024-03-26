use chrono::{DateTime, Utc};

pub struct Project{
    pub id: i64,
    pub name: String,
    pub encoded: String,
    pub created_at: DateTime<Utc>,
}

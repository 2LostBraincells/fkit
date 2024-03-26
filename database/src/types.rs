use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, AnyPool};

/// A bare-bones representation of a project
#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub struct RawProject {
    pub id: i64,
    pub name: String,
    pub encoded: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Project{
    pool: AnyPool,
    pub id: i64,
    pub name: String,
    pub encoded: String,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn from_raw(
        raw: RawProject,
        pool: AnyPool,
    ) -> Option<Project> {
        let created_at = DateTime::from_timestamp(raw.created_at, 0)?;

        Some(Project {
            pool,
            created_at,
            id: raw.id,
            name: raw.name,
            encoded: raw.encoded,
        })
    }
}

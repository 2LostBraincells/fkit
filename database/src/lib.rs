use chrono::{DateTime, Utc};
use sqlx::{migrate, AnyPool};

#[allow(unused)]
pub struct Database {
    pool: AnyPool,
}

#[allow(unused)]
pub struct Dataset {
    pool: AnyPool,
    id: i64,
    name: String,
    created_at: DateTime<Utc>,
}

#[allow(unused)]
pub struct Datastream {
    id: i64,
    data_type: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct RawDataset {
    id: i64,
    name: String,
    created_at: i64,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct RawDatastream {
    id: i64,
    name: String,
    dataset_id: i64,
    data_type: String,
    created_at: i64,
}

impl Database {
    pub async fn new(url: &str) -> Database {
        let pool = sqlx::Pool::connect(url).await.unwrap();

        migrate!("./migrations").run(&pool).await.unwrap();

        Database { pool }
    }
}

impl Database {
    pub async fn dataset(&self, name: &str) -> Dataset {
        let result: RawDataset = sqlx::query_as(r#"SELECT * FROM Dataset WHERE name = ?"#)
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        Dataset {
            pool: self.pool.clone(),
            id: result.id,
            name: result.name,
            created_at: DateTime::from_timestamp_millis(result.created_at).unwrap(),
        }
    }

    pub async fn datasets(&self) -> impl Iterator<Item = Dataset> + '_ {
        // Get all datasets as raw
        sqlx::query_as(r#"SELECT * FROM Dataset"#)
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .into_iter()
            // Convert to Dataset
            .map(|result: RawDataset| Dataset {
                pool: self.pool.clone(),
                id: result.id,
                name: result.name,
                created_at: DateTime::from_timestamp_millis(result.created_at).unwrap(),
            })
    }
}

impl Dataset {
    pub async fn datastreams(&self) -> impl Iterator<Item = Datastream> + '_ {
        // Get all datastreams as raw
        sqlx::query_as(r#"SELECT * FROM Datastream WHERE dataset_id = ?"#)
            .bind(self.id)
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .into_iter()
            .map(|result: RawDatastream| Datastream {
                id: result.id,
                data_type: result.data_type,
                name: result.name,
                created_at: DateTime::from_timestamp_millis(result.created_at).unwrap(),
            })
    }
}

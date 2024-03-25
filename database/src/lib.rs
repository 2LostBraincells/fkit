use chrono::NaiveDateTime;
use sqlx::{migrate, Pool, Sqlite, SqlitePool};
use std::sync::Arc;

#[allow(unused)]
pub struct Database<Driver>
where
    Driver: sqlx::Database,
{
    pool: Arc<Pool<Driver>>,
}

#[allow(unused)]
pub struct Dataset<Driver>
where
    Driver: sqlx::Database,
{
    pool: Arc<Pool<Driver>>,
    id: i64,
    name: String,
    created_at: NaiveDateTime,
}

#[allow(unused)]
pub struct Datastream {
    id: i64,
    data_type: String,
    name: String,
    created_at: NaiveDateTime,
}

#[allow(dead_code)]
struct RawDataset {
    id: i64,
    name: String,
    created_at: NaiveDateTime,
}

#[allow(dead_code)]
struct RawDatastream {
    id: i64,
    name: String,
    dataset_id: i64,
    data_type: String,
    created_at: NaiveDateTime,
}

impl Database<Sqlite> {
    pub async fn new(url: &str) -> Database<Sqlite> {
        let pool = SqlitePool::connect(url).await.unwrap();

        migrate!("./migrations").run(&pool).await.unwrap();

        Database {
            pool: Arc::new(pool),
        }
    }
}

impl<Driver: sqlx::Database> Database<Driver> {
    pub async fn dataset(&self, name: &str) -> Dataset<Driver> {
        let pool = *self.pool;
        let result = sqlx::query_as!(RawDataset, r#"SELECT * FROM Dataset WHERE name = ?"#, name)
            .fetch_one(pool)
            .await
            .unwrap();

        Dataset {
            pool: self.pool.clone(),
            id: result.id,
            name: result.name,
            created_at: result.created_at,
        }
    }

    pub async fn datasets(&self) -> impl Iterator<Item = Dataset<Driver>> + '_ {
        let pool = &*self.pool;
        // Get all datasets as raw
        sqlx::query_as!(RawDataset, r#"SELECT * FROM Dataset"#)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            // Convert to Dataset
            .map(|result| Dataset {
                pool: self.pool.clone(),
                id: result.id,
                name: result.name,
                created_at: result.created_at,
            })
    }
}

impl<Driver: sqlx::Database> Dataset<Driver> {
    pub async fn datastreams(&self) -> impl Iterator<Item = Datastream> + '_ {
        let pool = *self.pool;
        // Get all datastreams as raw
        sqlx::query_as!(
            RawDatastream,
            r#"SELECT * FROM Datastream WHERE dataset_id = ?"#,
            self.id
        )
        .fetch_all(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|result| Datastream {
            id: result.id,
            data_type: result.data_type,
            name: result.name,
            created_at: result.created_at,
        })
    }
}

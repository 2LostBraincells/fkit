use sqlx::{migrate, prelude::FromRow, AnyPool};
use types::Project;

pub mod types;

#[allow(unused)]
pub struct Database {
    pool: AnyPool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Database, sqlx::Error> {
        sqlx::any::install_default_drivers();
        let pool = sqlx::Pool::connect(url).await?;

        migrate!("./migrations").run(&pool).await?;

        Ok(Database { pool })
    }

    /// Get a list of all the projects in the database
    /// 
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory").await?;
    /// let projects = db.get_projects().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    /// [Project]s or sqlx error if the query failed 
    pub async fn get_projects(&self) -> Result<Vec<Project>, sqlx::Error> {
        #[derive(FromRow)]
        struct RawProject {
            pub id: i64,
            pub name: String,
            pub encoded: String,
            pub created_at: i64,
        }
        let projects = sqlx::query_as::<_, RawProject>("SELECT * FROM projects")
            .fetch_all(&self.pool)
            .await?;

        Ok(projects
            .into_iter()
            .map(|p| Project {
                id: p.id,
                name: p.name,
                encoded: p.encoded,
                created_at: chrono::DateTime::from_timestamp(p.created_at, 0)
                    .expect("Invalid Timestamp"),
            })
            .collect())
    }
}

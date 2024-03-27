use crate::project::{Project, RawProject};
use sqlx::{migrate, AnyPool};

/// Database for holding all project data and metadata
#[allow(unused)]
pub struct Database {
    /// generic sqlx connection pool
    pool: AnyPool,
}

impl Database {
    /// Shorthand for creating a new database connection.
    ///
    /// This will install all available drivers and run the migrations in `./migrations`
    ///
    /// # Arguments
    /// * `url` Url to the database
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(url: &str) -> Result<Database, sqlx::Error> {
        // Install all drivers and setup connection
        sqlx::any::install_default_drivers();
        let pool = sqlx::Pool::connect(url).await?;

        // Run migrations
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
        // Fetch and deserialize
        let projects: Vec<RawProject> = sqlx::query_as("SELECT * FROM projects")
            .fetch_all(&self.pool)
            .await?;

        // Convert from Raw to actual project
        Ok(projects
            .into_iter()
            .map(|p| Project::from_raw(p, self.pool.clone()).unwrap())
            .collect())
    }

    /// Get a specific project by name
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory").await?;
    /// let project = db.get_project("foo").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    /// [Project] or None if the project does not exist
    /// Error if the query failed
    pub async fn get_project(&self, name: &str) -> Result<Option<Project>, sqlx::Error> {
        // Fetch and deserialize
        let project: RawProject = sqlx::query_as("SELECT * FROM projects WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        // Convert from Raw to actual project
        Ok(Project::from_raw(project, self.pool.clone()))
    }
}

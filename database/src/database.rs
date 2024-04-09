use crate::{
    project::{Project, RawProject},
    utils::sql_encode,
};
use chrono::Utc;
use sqlx::{migrate, AnyPool};

/// Database for holding all project data and metadata
#[allow(unused)]
#[derive(Debug, Clone)]
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
    /// let db = Database::new("sqlite::memory:").await.expect("Database should be created");
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
    /// ```
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory:").await.expect("Database should be created");
    ///
    /// let projects = db.get_projects().await.expect("Getting all projects should work");
    /// assert_eq!(projects.len(), 0);
    ///
    /// db.create_project("foo").await.expect("Project foo should have been created");
    /// db.create_project("bar").await.expect("Project bar should have been created");
    ///
    /// let projects = db.get_projects().await.expect("Getting all projects should work");
    /// assert_eq!(projects.len(), 2);
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
    /// # Arguments
    /// * `name` - Name of the project to fetch
    ///
    /// # Examples
    /// ```
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory:").await.expect("Database should be created");
    ///
    /// db.create_project("foo").await.expect("Project should have been created");
    /// let project = db.get_project("foo").await.expect("Getting project should be successful");
    ///
    /// assert!(project.is_some());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory:").await.expect("Database should be created");
    /// let project = db.get_project("foo").await.expect("Getting project should be successful");
    ///
    /// assert!(project.is_none());
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

    /// Create a new project
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory:").await.expect("Database should be created");
    /// let project = db.create_project("foo").await.expect("Project should have been created");
    ///
    /// assert_eq!(project.name, "foo");
    /// assert_eq!(project.encoded, "foo");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_project(&self, name: &str) -> Result<Project, sqlx::Error> {
        // Encode the name
        let encoded = sql_encode(name).expect("Valid name");

        dbg!(&encoded);

        // Create table
        sqlx::query(dbg!(&format!("CREATE TABLE {} (timestamp INTEGER NOT NULL);", encoded)))
            .execute(&self.pool)
            .await?;

        let timestamp = Utc::now().timestamp();
        dbg!(timestamp);

        // Insert the project
        let project: RawProject = sqlx::query_as(
            r#"
            INSERT INTO projects (name, encoded_name, created_at) VALUES (?, ?, ?) RETURNING *
            "#,
        )
        .bind(name)
        .bind(encoded)
        .bind(timestamp)
        .fetch_one(&self.pool)
        .await?;

        // Convert from Raw to actual project
        Ok(Project::from_raw(project, self.pool.clone()).unwrap())
    }

    /// Get the pool for this database
    pub fn get_pool(&self) -> AnyPool {
        self.pool.clone()
    }
}

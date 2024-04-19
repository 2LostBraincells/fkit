use std::collections::HashMap;

use crate::{
    project::{Project, RawProject},
    utils::sql_encode,
};
use chrono::Utc;
use sqlx::{migrate, prelude::FromRow, AnyPool, Executor};

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
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:new_database?mode=memory").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(url: &str) -> Result<Database, sqlx::Error> {
        // Install all drivers and setup connection
        sqlx::any::install_default_drivers();
        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(99)
            .idle_timeout(None)
            .connect(url)
            .await?;

        // Run migrations
        migrate!("./migrations").run(&pool).await?;

        Ok(Database { pool })
    }

    /// Get a list of all the projects in the database
    ///
    /// # Examples
    /// ```
    /// # use database::Database;
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:foo?mode=memory").await?;
    ///
    /// let projects = db.get_projects().await?;
    /// assert_eq!(projects.len(), 0);
    ///
    /// db.create_project("foo").await?;
    /// db.create_project("bar").await?;
    ///
    /// let projects = db.get_projects().await?;
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
            .map(|p| Project::from_raw(p, self.pool.clone()).expect("project should be valid"))
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
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:foo?mode=memory").await?;
    ///
    /// db.create_project("foo").await?;
    /// let project = db.get_project("foo").await?;
    ///
    /// assert!(project.is_some());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use database::Database;
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:foo?mode=memory").await?;
    /// let project = db.get_project("foo").await?;
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
        let project: RawProject = match sqlx::query_as("SELECT * FROM projects WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await
        {
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(e) => return Err(e),
            Ok(p) => p,
        };

        // Convert from Raw to actual project
        Ok(Project::from_raw(project, self.pool.clone()))
    }

    /// Create a new project
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:foo?mode=memory").await?;
    /// let project = db.create_project("foo").await?;
    ///
    /// assert_eq!(project.name, "foo");
    /// assert_eq!(project.encoded, "foo");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_project(&self, name: &str) -> Result<Project, sqlx::Error> {
        // Encode the name
        let encoded = sql_encode(name).unwrap_or_else(|e| e);

        dbg!(&encoded);

        // Create table
        self.create_project_table(&encoded).await?;

        let timestamp = Utc::now().timestamp();
        dbg!(timestamp);

        // Insert the project
        let project = self.insert_project(name, &encoded, timestamp).await?;

        // Convert from Raw to actual project
        Ok(Project::from_raw(project, self.pool.clone()).unwrap())
    }

    /// Creates a base table with a given name. Name is not sanitized so please do that before
    /// calling the function.
    async fn create_project_table(&self, encoded_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "CREATE TABLE {} (timestamp INTEGER NOT NULL);",
            encoded_name
        ))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert a project inte the database.
    async fn insert_project(
        &self,
        name: &str,
        encoded: &str,
        timestamp: i64,
    ) -> Result<RawProject, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO projects (name, encoded_name, created_at) VALUES (?, ?, ?) RETURNING *
            "#,
        )
        .bind(name)
        .bind(encoded)
        .bind(timestamp)
        .fetch_one(&self.pool)
        .await
    }

    /// Retrieves the project id of a project, given the name, from the database.
    pub async fn get_project_id(&self, project: &str) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT id FROM projects WHERE name = ?
            "#,
        )
        .bind(project)
        .fetch_one(&self.pool)
        .await
    }
}

#[cfg(test)]
pub mod methods {
    use crate::{project::Project, Database};

    #[tokio::test]
    async fn create_memory_database() {
        create_mem_db("create_db").await;
    }

    #[tokio::test]
    async fn create_project() {
        let db = create_mem_db("create_project").await;
        let project = db.create("foo").await;

        assert_eq!(project.name, "foo");
        assert_eq!(project.encoded, "foo");
    }

    #[tokio::test]
    #[allow(clippy::disallowed_names)]
    async fn get_project() {
        let db = create_mem_db("get_project").await;

        db.create("foo").await;

        let foo = db.get("foo").await;
        let roo = db.get("bar").await;

        assert!(foo.is_some());
        assert!(roo.is_none());
    }

    #[tokio::test]
    async fn get_projects() {
        let db = create_mem_db("get_projects").await;

        let projects = db.get_all().await;

        assert_eq!(projects.len(), 0);

        db.create("foo").await;
        db.create("bar").await;

        let projects = db.get_all().await;

        assert_eq!(projects.len(), 2);
    }

    pub async fn create_mem_db(name: &str) -> Database {
        Database::new(&format!("sqlite:file:{}?mode=memory", name))
            .await
            .expect("Database should be created")
    }

    impl Database {
        async fn create(&self, name: &str) -> Project {
            self.create_project(name)
                .await
                .expect("Project should have been created")
        }

        async fn get(&self, name: &str) -> Option<Project> {
            self.get_project(name)
                .await
                .expect("Project should have been fetched")
        }

        async fn get_all(&self) -> Vec<Project> {
            self.get_projects()
                .await
                .expect("Projects should have been fetched")
        }
    }
}

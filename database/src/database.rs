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
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:foo?mode=memory").await?;
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
        let encoded = sql_encode(name).expect("Valid name");

        dbg!(&encoded);

        // Create table
        sqlx::query(dbg!(&format!(
            "CREATE TABLE {} (timestamp INTEGER NOT NULL);",
            encoded
        )))
        .execute(&self.pool)
        .await?;

        let timestamp = Utc::now().timestamp();
        dbg!(timestamp);

        // Insert the project
        let project: RawProject = dbg!(sqlx::query_as(
                    r#"
                    INSERT INTO projects (name, encoded_name, created_at) VALUES (?, ?, ?) RETURNING *
                    "#,
                )
                .bind(name)
                .bind(encoded)
                .bind(timestamp)
                .fetch_one(&self.pool)
                .await?);

        // Convert from Raw to actual project
        Ok(Project::from_raw(project, self.pool.clone()).unwrap())
    }

    /// Get the pool for this database
    pub fn get_pool(&self) -> AnyPool {
        self.pool.clone()
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
        let project = cre_proj(&db, "foo").await;

        assert_eq!(project.name, "foo");
        assert_eq!(project.encoded, "foo");
    }

    #[tokio::test]
    #[allow(clippy::disallowed_names)]
    async fn get_project() {
        let db = create_mem_db("get_project").await;

        cre_proj(&db, "foo").await;

        let foo = get_proj(&db, "foo").await;
        let roo = get_proj(&db, "bar").await;

        assert!(foo.is_some());
        assert!(roo.is_none());
    }

    #[tokio::test]
    async fn get_projects() {
        let db = create_mem_db("get_projects").await;

        let projects = get_all_projs(&db).await;

        assert_eq!(projects.len(), 0);

        cre_proj(&db, "foo").await;
        cre_proj(&db, "bar").await;

        let projects = get_all_projs(&db).await;

        assert_eq!(projects.len(), 2);
    }

    pub async fn create_mem_db(name: &str) -> Database {
        Database::new(&format!("sqlite:file:{}?mode=memory", name))
            .await
            .expect("Database should be created")
    }

    pub async fn cre_proj(db: &Database, name: &str) -> Project {
        db.create_project(name)
            .await
            .expect("Project should have been created")
    }

    pub async fn get_proj(db: &Database, name: &str) -> Option<Project> {
        db.get_project(name)
            .await
            .expect("Project should have been fetched")
    }

    pub async fn get_all_projs(db: &Database) -> Vec<Project> {
        db.get_projects()
            .await
            .expect("Projects should have been fetched")
    }
}

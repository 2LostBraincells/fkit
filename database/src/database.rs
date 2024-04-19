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

#[derive(Debug, FromRow, Clone)]
pub struct RawColumn {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub encoded_name: String,
    pub created_at: i64,
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
    /// let db = Database::new("sqlite:file:new_database?mode=memory").await.expect("Database should be created");
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
    /// let db = Database::new("sqlite:file:foo?mode=memory").await.expect("Database should be created");
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
    /// let db = Database::new("sqlite:file:foo?mode=memory").await.expect("Database should be created");
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
    /// let db = Database::new("sqlite:file:foo?mode=memory").await.expect("Database should be created");
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
    /// let db = Database::new("sqlite:file:foo?mode=memory").await.expect("Database should be created");
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

    #[inline]
    /// Get the pool for this database
    pub fn get_pool(&self) -> AnyPool {
        self.pool.clone()
    }

    /// Creates a new column for a given project with a given name
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:zoo?mode=memory").await.expect("Database should be created");
    /// let project = db.create_project("name_of_valid_project").await.expect("Project should have been created");
    /// db.create_column("name_of_valid_project", "bar").await.expect("Column should have been created");
    /// db.create_column("not_a_valid_project", "bar").await.expect_err("Column should not have been created");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_column(&self, project: &str, name: &str) -> Result<(), sqlx::Error> {
        self.add_column(project, name).await?;
        self.insert_column(project, name).await
    }

    /// Alters the table of a given project to add a new column with the given name
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:zoo?mode=memory").await.expect("Database should be created");
    /// let project = db.create_project("foo").await.expect("Project should have been created");
    /// db.add_column("foo", "bar").await.expect("Column should have been created");
    /// # Ok(())
    /// # }
    /// ````
    /// -- Table schema is now:
    /// CREATE TABLE foo (timestamp INTEGER NOT NULL, bar TEXT);
    pub async fn add_column(&self, project: &str, name: &str) -> Result<(), sqlx::Error> {
        let encoded_name = dbg!(sql_encode(name).unwrap_or_else(|e| e));
        let encoded_project = dbg!(sql_encode(project).unwrap_or_else(|e| e));

        sqlx::query(&format!(
            r#"
            ALTER TABLE {} ADD COLUMN {} TEXT
            "#,
            &encoded_project, &encoded_name
        ))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Inserts the column into the columns table of the database
    ///
    /// # Examples
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:roo?mode=memory")
    ///     .await
    ///     .expect("Database should be created");
    ///
    /// db.create_project("foo")
    ///     .await
    ///     .expect("Project should have been created");
    ///
    /// db.insert_column("foo", "bar")
    ///     .await
    ///     .expect("Column should have been inserted");
    ///
    /// db.insert_column("bar", "baz")
    ///     .await
    ///     .expect_err("Column should not have been created");
    /// # Ok(())
    /// # }
    pub async fn insert_column(&self, project: &str, name: &str) -> Result<(), sqlx::Error> {
        let encoded_name = dbg!(sql_encode(name).unwrap_or_else(|e| e));
        let project_id = dbg!(self.get_project_id(project).await?);
        let created_at = Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO columns 
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(project_id)
        .bind(name)
        .bind(encoded_name)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
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

    pub async fn add_datapoint(values: HashMap<String, String>) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod methods {
    use crate::{project::Project, Database};

    #[tokio::test]
    async fn create_memory_database() {
        create_mem_db("create_db").await;
    }

    #[tokio::test]
    async fn create_project() {
        let db = create_mem_db("create_project").await;
        let project = create(&db, "foo").await;

        assert_eq!(project.name, "foo");
        assert_eq!(project.encoded, "foo");
    }

    #[tokio::test]
    #[allow(clippy::disallowed_names)]
    async fn get_project() {
        let db = create_mem_db("get_project").await;

        create(&db, "foo").await;

        let foo = get(&db, "foo").await;
        let roo = get(&db, "bar").await;

        assert!(foo.is_some());
        assert!(roo.is_none());
    }

    #[tokio::test]
    async fn get_projects() {
        let db = create_mem_db("get_projects").await;

        let projects = get_all(&db).await;

        assert_eq!(projects.len(), 0);

        create(&db, "foo").await;
        create(&db, "bar").await;

        let projects = get_all(&db).await;

        assert_eq!(projects.len(), 2);
    }

    async fn create_mem_db(name: &str) -> Database {
        Database::new(&format!("sqlite:file:{}?mode=memory", name))
            .await
            .expect("Database should be created")
    }

    async fn create(db: &Database, name: &str) -> Project {
        db.create_project(name)
            .await
            .expect("Project should have been created")
    }

    async fn get(db: &Database, name: &str) -> Option<Project> {
        db.get_project(name)
            .await
            .expect("Project should have been fetched")
    }

    async fn get_all(db: &Database) -> Vec<Project> {
        db.get_projects()
            .await
            .expect("Projects should have been fetched")
    }
}

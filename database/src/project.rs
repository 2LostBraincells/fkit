use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, AnyPool};

use crate::utils;

#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ProjectBuilder {
    name: String,
    encoded: String,
    created_at: i64,
}

/// A bare-bones representation of a project
#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub struct RawProject {
    pub name: String,
    #[sqlx(rename = "encoded_name")]
    pub encoded: String,
    pub created_at: i64,
    pub id: i64,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub struct RawColumn {
    pub name: String,
    pub encoded: String,
    pub project_id: i64,
    pub column_type: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Project {
    pool: AnyPool,
    pub id: i64,
    pub name: String,
    pub encoded: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    Text,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub encoded: String,
    pub project_id: i64,
    pub column_type: DataType,
    pub created_at: DateTime<Utc>,
}

impl ProjectBuilder {
    /// Creates a new project builder with the given name.
    pub async fn new(name: String) -> ProjectBuilder {
        ProjectBuilder {
            name,
            encoded: String::new(),
            created_at: Utc::now().timestamp(),
        }
    }

    /// Build the project. This involves inserting the project into the database as well as
    /// fetching the project from the database.
    pub async fn build(self, pool: AnyPool) -> Result<Project, sqlx::Error> {
        let project = sqlx::query_as::<_, RawProject>(
            r#"
            INSERT INTO projects (name, encoded, created_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(self.name)
        .bind(self.encoded)
        .bind(self.created_at)
        .fetch_one(&pool)
        .await?;

        Project::from_raw(project, pool).ok_or(sqlx::Error::Decode(Box::new(
            sqlx::error::Error::RowNotFound,
        )))
    }
}

impl Project {
    pub fn from_raw(raw: RawProject, pool: AnyPool) -> Option<Project> {
        let created_at =
            DateTime::from_timestamp(raw.created_at, 0).expect("Timestamp should be valid");

        Some(Project {
            pool,
            created_at,
            id: raw.id,
            name: raw.name,
            encoded: raw.encoded,
        })
    }

    /// Get all columns for this project
    ///
    /// # Example
    /// ```rust
    /// # use database::Database;
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:get_columns?mode=memory").await?;
    ///
    /// db.create_project("foo").await?;
    /// let foo = db.get_project("foo").await?.unwrap();
    ///
    /// let columns = foo.get_columns().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn get_columns(&self) -> Result<Vec<Column>, sqlx::Error> {
        // Fetch and deserialize
        let raw: Vec<RawColumn> = sqlx::query_as(
            r#"
            SELECT * FROM columns WHERE project_id = $1
            "#,
        )
        .bind(self.id)
        .fetch_all(&self.pool)
        .await?;

        let mut columns = Vec::with_capacity(raw.len());

        // Convert from Raw
        for column in raw {
            match Column::from_raw(column) {
                Some(c) => columns.push(c),
                None => panic!("Failed to convert column"),
            }
        }
        Ok(columns)
    }

    /// Add a new data column to the project
    ///
    /// # Arguments
    /// * `name` - Name of the column
    /// * `type` - Type of the column
    pub async fn create_column(
        &self,
        _name: &str,
        _encoded: &str,
        _column_type: DataType,
    ) -> Result<Column, sqlx::Error> {
        todo!();
    }

    /// Add column to the columns table
    async fn add_column_to_index(
        &self,
        name: &str,
        _encoded: &str,
        _column_type: DataType,
    ) -> Result<Column, sqlx::Error> {
        let now: i32 = Utc::now().timestamp().try_into().unwrap();
        sqlx::query_as(
            r#"
            INSERT INTO columns (name, encoded, column_type, project_id, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(_encoded)
        .bind(_column_type.to_sql())
        .bind(self.id)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map(Column::from_raw)
        .map(|c| c.expect("Failed to convert column"))
    }

    async fn add_column_to_table(_column: Column) -> Result<(), sqlx::Error> {
        todo!()
    }
}

impl Column {
    /// Convert a RawColumn to a Column
    ///
    /// # Returns
    /// Some(Column) if the conversion was successful
    /// None if the conversion failed
    pub fn from_raw(raw: RawColumn) -> Option<Column> {
        let created_at = DateTime::from_timestamp(raw.created_at, 0)?;

        Some(Column {
            column_type: DataType::from_sql(&raw.column_type)?,
            name: raw.name,
            encoded: raw.encoded,
            project_id: raw.project_id,
            created_at,
        })
    }
}

impl DataType {
    /// Convert the data type to a string for SQL
    ///
    /// # Example
    /// ```rust
    /// # use database::project::DataType;
    /// let data_type = DataType::Text;
    /// assert_eq!(data_type.to_sql(), "TEXT");
    /// ```
    ///
    /// # Returns
    /// A sql type as a string
    pub fn to_sql(&self) -> &str {
        match self {
            DataType::Text => "TEXT",
            DataType::Unknown => "BLOB",
        }
    }

    /// Convert a string from SQL to a data type
    ///
    /// If the string is not a valid data type, return None
    ///
    /// # Example
    /// ```rust
    /// # use database::project::DataType;
    /// let data_type = DataType::from_sql("TEXT");
    /// assert_eq!(data_type, Some(DataType::Text));
    /// ```
    ///
    /// ```rust
    /// # use database::project::DataType;
    /// let data_type = DataType::from_sql("NOT_A_TYPE");
    /// assert_eq!(data_type, None);
    /// ```
    ///
    /// # Returns
    /// DataType if the string is a valid sql type or None otherwise
    pub fn from_sql(s: &str) -> Option<DataType> {
        match s {
            "TEXT" => Some(DataType::Text),
            "BLOB" => Some(DataType::Unknown),
            _ => None,
        }
    }
}

#[cfg(test)]
mod methods {
    use crate::{
        database::methods::{cre_proj, create_mem_db},
        project::DataType,
        utils::sql_encode,
    };

    use super::{Column, Project};

    #[tokio::test]
    async fn add_column_to_index() {
        let db = create_mem_db("add_column_to_index").await;
        let project = cre_proj(&db, "foo").await;
        project
            .add_column_to_index("boo", "boo", DataType::Text)
            .await
            .expect("Adding column to index should work");

        let column = project
            .get_columns()
            .await
            .expect("Columns should have been fetched")
            .pop()
            .expect("Column should exist");

        assert_eq!(column.name, "boo");
    }
}

use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, AnyPool};

#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub struct ProjectBuilder {
    name: String,
    encoded: String,
    created_at: i64,
}

/// A bare-bones representation of a project
#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub struct RawProject {
    pub id: i64,
    pub name: String,
    #[sqlx(rename = "encoded_name")]
    pub encoded: String,
    pub created_at: i64,
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
        let created_at = DateTime::from_timestamp(raw.created_at, 0)?;

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
    /// # tokio_test::block_on(test());
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite::memory").await?;
    /// let foo = db.get_project("foo").await?.unwrap();
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

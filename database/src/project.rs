use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, AnyPool};

use crate::utils::sql_encode;

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

    /// Creates a new column for a given project with a given name
    ///
    /// # Examples
    /// ```rust
    /// # use database::{Database, project::DataType};
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:create_column?mode=memory").await?;
    /// let project = db.create_project("foo").await?;
    ///
    /// project.create_column("bar", DataType::Text).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_column(
        &self,
        name: &str,
        column_type: DataType,
    ) -> Result<Option<Column>, sqlx::Error> {
        let encoded_name = sql_encode(name).unwrap_or_else(|e| e);

        self.add_column(&encoded_name, column_type).await?;
        let raw_column = self.insert_column(name, &encoded_name, column_type).await?;

        Ok(Column::from_raw(raw_column))
    }

    /// Alters the table of a given project to add a new column with the given name
    ///
    /// # Examples
    /// ```rust
    /// # use database::{Database, project::DataType};
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error>{
    /// let db = Database::new("sqlite:file:create_column?mode=memory").await?;
    /// let project = db.create_project("foo").await?;
    ///
    /// project.add_column("bar", DataType::Text).await?;
    /// # Ok(())
    /// # }
    /// ````
    /// -- Table schema is now:
    /// CREATE TABLE foo (__timestamp__ INTEGER NOT NULL, bar TEXT);
    pub async fn add_column(
        &self,
        encoded_name: &str,
        column_type: DataType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            r#"
            ALTER TABLE {} ADD COLUMN {} {}
            "#,
            &self.encoded,
            &encoded_name,
            column_type.to_sql()
        ))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Inserts the column into the columns table of the database
    ///
    /// # Examples
    /// ```rust
    /// # use database::{Database, project::DataType};
    /// # tokio_test::block_on(test()).unwrap();
    /// # async fn test() -> Result<(), sqlx::Error> {
    /// let db = Database::new("sqlite:file:create_column?mode=memory").await?;
    /// let project = db.create_project("foo").await?;
    ///
    /// project.insert_column("bar", "bar", DataType::Text).await?;
    /// # Ok(())
    /// # }
    pub async fn insert_column(
        &self,
        name: &str,
        encoded_name: &str,
        column_type: DataType,
    ) -> Result<RawColumn, sqlx::Error> {
        let created_at = Utc::now().timestamp();
        sqlx::query_as(
            r#"
            INSERT INTO columns 
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(self.id)
        .bind(name)
        .bind(encoded_name)
        .bind(column_type.to_sql())
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Adds a datapoint to the project
    pub async fn add_datapoint(&self, data: HashMap<String, String>) -> Result<(), sqlx::Error> {
        let mut keys = Vec::with_capacity(data.len());
        let mut values = Vec::with_capacity(data.len());

        keys.push("__timestamp__".to_string());
        values.push(Utc::now().timestamp().to_string());

        for (key, value) in data.iter() {
            keys.push(key.to_string());
            values.push(value.to_string());
        }

        // make sure all of the columns exist
        let columns = self.get_or_create_columns(&keys).await?;

        let query = self.generate_query(
            &columns
                .iter()
                .map(|c| c.encoded.clone())
                .collect::<Vec<String>>(),
        );

        values
            .iter()
            .fold(sqlx::query(&query), |query, value| query.bind(value))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Generate sql query for inserting data into the project table
    fn generate_query(&self, encoded_names: &[String]) -> String {
        format!(
            r#"
            INSERT INTO {} ({})
            VALUES ({})
            "#,
            self.encoded,
            encoded_names.join(","),
            encoded_names
                .iter()
                .map(|_| "?")
                .collect::<Vec<&str>>()
                .join(",")
        )
    }

    /// Will verify that all the given keys correspond with a column in the database, creating any
    /// columns that do not exist. Returning an array of columns, guaranteed to be in the same
    /// order as the keys
    async fn get_or_create_columns(&self, keys: &[String]) -> Result<Vec<Column>, sqlx::Error> {
        // Get existing columns
        let pre = self.get_columns().await?;
        let mut columns = HashMap::with_capacity(pre.len());

        pre.into_iter().for_each(|c| {
            columns.insert(c.name.clone(), c);
        });

        let mut result = Vec::with_capacity(keys.len());

        for key in keys {
            match columns.remove(key) {
                Some(c) => result.push(c),
                None => {
                    let column = self.create_column(key, DataType::Text).await?;
                    result.push(column.expect("Parsing should work"));
                }
            }
        }

        Ok(result)
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
    use crate::{database::methods::create_mem_db, project::DataType};

    use super::{Column, Project};

    #[tokio::test]
    async fn create_column() {
        let db = create_mem_db("create_column").await;
        let project = db.create("foo").await;
        let column = project.create("boo").await;

        assert_eq!(column.name, "boo");
        assert_eq!(column.encoded, "boo");
    }

    impl Project {
        pub async fn create(&self, name: &str) -> Column {
            self.create_column(name, DataType::Text)
                .await
                .expect("Column should be created")
                .expect("Parsing column shouldn't fail")
        }
    }
}

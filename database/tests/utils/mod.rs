use std::path::PathBuf;

use database::{
    project::{Column, DataType, Project},
    Database,
};

pub async fn create_mem_db(name: &str) -> Database {
    Database::new(&format!("sqlite:file:{}?mode=memory", name))
        .await
        .expect("Database should be created")
}

pub async fn create_file_db(path: PathBuf) -> Database {
    Database::new(&format!("sqlite:{}", path.to_string_lossy()))
        .await
        .expect("Database should be created")
}

pub async fn cre_proj(db: &Database, name: &str) -> Project {
    db.create_project(name)
        .await
        .expect("Project should have been created")
}

pub async fn get(db: &Database, name: &str) -> Option<Project> {
    db.get_project(name)
        .await
        .expect("Project should have been fetched")
}

pub async fn get_all(db: &Database) -> Vec<Project> {
    db.get_projects()
        .await
        .expect("Projects should have been fetched")
}

pub async fn cre_col(project: &Project, name: &str) -> Column {
    project
        .create_column(name, DataType::Text)
        .await
        .expect("Column should be created")
}

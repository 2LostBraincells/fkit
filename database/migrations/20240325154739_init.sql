-- Add migration script here
CREATE TABLE IF NOT EXISTS projects (
    name STRING UNIQUE NOT NULL,
    encoded_name STRING UNIQUE NOT NULL,
    id INTEGER UNIQUE NOT NULL PRIMARY KEY AUTOINCREMENT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS columns (
    project_id INTEGER NOT NULL,
    name STRING NOT NULL,
    encoded_name STRING NOT NULL,
    created_at INTEGER NOT NULL,

    FOREIGN KEY (project_id) REFERENCES project(id)
);

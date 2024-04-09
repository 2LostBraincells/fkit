use std::{error::Error, ops::Deref, path::PathBuf};

use config_rs::{Config, ConfigError, File};
use serde::Deserialize;

use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseUrl {
    raw: String,
    scheme: String,
    location: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Schema {
    Sqlite,
    Postgres,
    Mysql,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    database: DatabaseConfig,
    server: Option<ServerConfig>,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    port: Option<u16>,
}

impl Settings {
    pub fn load(path: PathBuf) -> Result<Self, ConfigError> {
        let settings: Settings = Config::builder()
            .add_source(File::with_name(path.to_str().unwrap()))
            .build()?
            .try_deserialize()?;

        Ok(settings)
    }

    pub fn get_database_url(&self) -> DatabaseUrl {
        let raw = self.database.url.clone();
        let parts = raw.split("://").collect::<Vec<&str>>();
        DatabaseUrl {
            scheme: parts[0].to_string(),
            location: parts[1].to_string(),
            raw,
        }
    }

    pub fn get_server_port(&self) -> Option<u16> {
        self.server.as_ref().and_then(|s| s.port)
    }
}

impl DatabaseUrl {
    pub fn get_scheme(&self) -> Schema {
        match self.scheme.as_str() {
            "sqlite" => Schema::Sqlite,
            "postgres" => Schema::Postgres,
            "mysql" => Schema::Mysql,
            _ => panic!("Unsupported database schema"),
        }
    }

    pub fn get_location(&self) -> &str {
        &self.location
    }

    pub fn get_as_str(&self) -> &str {
        &self.raw
    }

    pub fn change_url<E>(&mut self, url: E)
    where
        E: Into<String>,
    {
        self.raw = url.into();
        let parts = self.raw.split("://").collect::<Vec<&str>>();
        self.scheme = parts[0].to_string();
        self.location = parts[1].to_string();
    }
}

pub fn generate_default_config() -> Result<String, Box<dyn Error>> {
    let dir_name = utils::current_directory_name()?;

    Ok(format!(
        r#"
[database]
url = "sqlite://./{}.db"
"#,
        dir_name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_url() {
        let settings = Settings {
            database: DatabaseConfig {
                url: "sqlite://./test.db".to_string(),
            },
            server: None,
        };

        let url = settings.get_database_url();
        assert_eq!(url.raw, "sqlite://./test.db");
        assert_eq!(url.scheme, "sqlite");
        assert_eq!(url.location, "./test.db");
    }
}

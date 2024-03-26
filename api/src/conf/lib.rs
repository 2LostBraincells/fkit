use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Settings {
    database: Option<DatabaseConfig>,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
}

impl Settings {
    pub fn new(path: Option<&str>) -> Result<Self, ConfigError> {
        let settings: Settings = Config::builder()
            .add_source(File::with_name(path.unwrap_or("fkit.toml")))
            .build()?
            .try_deserialize()?;

        Ok(settings)
    }

    pub fn get_database_url(&self) -> Option<&str> {
        self.database.as_ref().map(|db| db.url.as_str())
    }
}

use clap::{Parser, Subcommand};
use config::Settings;
use database::Database;
use std::{
    collections::HashMap,
    error::Error,
    path::PathBuf,
};

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

mod config;
mod utils;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {},
    Run {
        #[clap(short, long)]
        config: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Some(Command::Init {}) => {
            check_config_file()?;
        }
        Some(Command::Run { config }) => {
            run(config).await?;
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}

async fn run(config_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("fkit.toml"));
    let config = load_config_file(config_path)?;

    let database_url = config.get_database_url();
    let database_path = PathBuf::from(database_url.get_location());

    check_database_file(database_path)?;

    let database = Database::new(database_url.get_as_str()).await?;

    let routes = Router::new()
        .route("/new/:project", get(create_project))
        .route("/add/*path", get(catch_all_text));

    let app = Router::new().nest("/", routes).with_state(database);

    println!("Listening on: http://localhost:3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn catch_all_text(
    Path(project): Path<String>,
    Query(data): Query<HashMap<String, String>>,
) -> String {
    let mut response = format!("Project: {}\n", project);

    for (key, value) in data {
        let entry = format!("{}: {}\n", key, value);

        response.push_str(&entry)
    }

    response
}

async fn create_project(Path(project): Path<String>, State(database): State<Database>) -> String {
    format!("Project: {}", project)
}

/// Will check that the config file exists in the current directory and create it if it doesnt,
/// populating it with the default config.
fn check_config_file() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from("fkit.toml");
    if config_path.exists() {
        return Ok(());
    }

    std::fs::File::create(&config_path).unwrap();
    std::fs::write(&config_path, config::generate_default_config()?)?;

    Ok(())
}

fn check_database_file(database_path: PathBuf) -> Result<(), Box<dyn Error>> {
    if database_path.exists() {
        println!("Database exists");
        return Ok(());
    }

    dbg!(database_path.clone());
    std::fs::File::create(&database_path).unwrap();

    Ok(())
}

fn load_config_file(file_path: PathBuf) -> Result<Settings, Box<dyn Error>> {
    let settings = Settings::load(file_path)?;
    Ok(settings)
}

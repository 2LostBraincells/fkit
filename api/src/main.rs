use clap::{Parser, Subcommand};
use config::Settings;
use database::{project::ProjectBuilder, Database};
use std::{collections::HashMap, error::Error, path::PathBuf};

use axum::{
    extract::{Path, Query, State},
    routing::post,
    Router,
};
use tokio::net::TcpListener;

mod config;
mod utils;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
    #[clap(long)]
    config_help: bool,
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
            // if help {
            //     println!("Runs the fkit server with the given config file.");
            //     println!("If no config file is provided the program will fail.");
            //     println!("A standard config file can be created with \"fkit init\", but it can also be created manually.");
            // }
            run(config).await?;
        }
        None => {
            if args.config_help {
                println!("database url should be supplied by your database provider.");
                println!("If your are using the default sqlite, the url should be in the format: sqlite://path/to/database.db");
                println!("this path is usually absolute unless you specify the current directory with ./");
            } else {
                println!("No command provided. Run with --help for more information.");
            }
        }
    }

    Ok(())
}

async fn run(config_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("fkit.toml"));
    let config = load_config_file(config_path)?;

    let database_url = config.get_database_url();

    check_database_file(database_url.get_location().into())?;

    let database = Database::new(database_url.get_as_str()).await?;

    let routes = Router::new()
        .route("/new/:project", post(create_project))
        .route("/add/*path", post(catch_all_text));

    let app = Router::new().nest("/", routes).with_state(database);

    let port = config.get_server_port().unwrap_or(3000);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("Listening on: http://localhost:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn catch_all_text(
    Path(project): Path<String>,
    Query(data): Query<HashMap<String, String>>,
) -> String {
    let mut response = format!("Project: {}\n", project);
    let mut datapoint = HashMap::new();

    for (key, value) in data {
        let entry = format!("{}: {}\n", key, value);
        datapoint.insert(key, value);

        response.push_str(&entry)
    }

    response
}

/// Creates a new project and inserts it into the database along with a corresponding table.
async fn create_project(Path(project): Path<String>, State(database): State<Database>) -> String {
    if project.contains('/') {
        return "Project name cannot contain a '/'".to_string();
    }

    database.create_project(&project).await.unwrap();

    format!("{:?}", project)
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

/// Will check that the database file exists and create it if it doesnt.
/// The database file path is extracted from the config file.
fn check_database_file(database_path: PathBuf) -> Result<(), Box<dyn Error>> {
    if database_path.exists() {
        println!("Database exists");
        return Ok(());
    }

    std::fs::File::create(&database_path).inspect_err(|e| {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                println!("Could not create the database file. Ensure the database url is correct in the config file.");
                println!("For explanations on the config, run \"fkit --config-help\"");
            }
            _ => {
                println!("Error creating database file: {:?}", e);
            }
        }
    })?;
    Ok(())
}

/// Loads the config file from the given path and returns the settings.
fn load_config_file(file_path: PathBuf) -> Result<Settings, Box<dyn Error>> {
    let settings = Settings::load(file_path)?;
    Ok(settings)
}

use clap::{Parser, Subcommand};
use config::AppConfig;
use database::Database;
use std::{collections::HashMap, error::Error, path::PathBuf};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Result},
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
                println!("If you are using the default sqlite, the url should be in the format: sqlite://path/to/database.db");
                println!("this path is usually absolute unless specified with sqlite://./path/to/database.db");
            } else {
                println!("No command provided. Run with --help for more information.");
            }
        }
    }

    Ok(())
}

async fn run(config_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    // Load the config file
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("fkit.toml"));
    let config = AppConfig::load(config_path)?;

    // Make sure the database file exists and open the database
    let database_url = config.get_database_url();
    check_database_file(database_url.get_location().into())?;
    let database = Database::new(database_url.get_as_str()).await?;

    // Create the routes
    let routes = Router::new()
        .route("/new/:project", post(create_project))
        .route("/:project", post(add_datapoint))
        .route("/:project/columns", post(define_columns));

    // Create the app
    let app = Router::new().nest("/", routes).with_state(database);

    // Create the serber
    let port = config.get_server_port().unwrap_or(3000);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    // Start the server
    println!("Listening on: http://localhost:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Catches the keys and values from the query string and returns them in a formatted string.
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

async fn add_datapoint(
    Path(project): Path<String>,
    Query(data): Query<HashMap<String, String>>,
    State(database): State<Database>,
) -> Result<String>{
    let project = match database.get_project(&project).await.map_err(|e| format!("Error: {:?}", e).into_response())? {
        None => {
            println!("Project not found, creating new: {}", project);
            database.create_project(&project).await.map_err(|e| format!("Error: {:?}", e).into_response())?
        },
        Some(p) => p,
    };

    let mut datapoint = HashMap::new();
    for (key, value) in data {
        datapoint.insert(key, value);
    }

    project.add_datapoint(datapoint).await.map_err(|e| format!("Error: {:?}", e).into_response())?;

    Ok("Success".to_string())
}

/// Creates a new project and inserts it into the database along with a corresponding table.
async fn create_project(Path(project): Path<String>, State(database): State<Database>) -> String {
    if project.contains('/') {
        return "Project name cannot contain a '/'".to_string();
    }

    println!("Creating new project: {}", project);
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

    let write_res = std::fs::File::create(&database_path);

    if let Err(e) = &write_res {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                println!("Could not create the database file. Ensure the database url is correct in the config file.");
                println!("For explanations on the config, run \"fkit --config-help\"");
            }
            _ => {
                println!("Error creating database file: {:?}", e);
            }
        }
    };

    write_res?;

    Ok(())
}

async fn define_columns(
    Path(project): Path<String>,
    State(database): State<Database>,
    Query(query): Query<HashMap<String, String>>,
) -> String {
    let project = database.get_project(&project).await.unwrap().unwrap();

    "bozo".to_string()
}

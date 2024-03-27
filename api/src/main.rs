use clap::{Parser, Subcommand};
use database::Database;
use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

mod config;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {},
    Run {},
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Some(Command::Init {}) => {
            check_config_file();
        }
        Some(Command::Run {}) => {
            run().await.unwrap();
        }
        None => {
            println!("No command provided");
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/new/:project", get(create_project))
        .route("/add/*path", get(catch_all_text));

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

async fn create_project(Path(project): Path<String>) -> String {
    format!("Project: {}", project)
}

fn check_config_file() {
    let config_path = std::path::PathBuf::from("fkit.toml");
    if config_path.exists() {
        return;
    }

    std::fs::File::create(&config_path).unwrap();
    std::fs::write(&config_path, config::DEFAULT_CONFIG).unwrap();
}

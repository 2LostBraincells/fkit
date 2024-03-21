use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/add/*path", get(catch_all_text));

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
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

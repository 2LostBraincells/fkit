use prisma::new_client;

use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpListener;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};

mod prisma;

#[tokio::main]
async fn main() {
    let db = Arc::new(new_client().await.unwrap());

    let app = Router::new()
        .route("/add/*path", get(catch_all_text))
        .with_state(db);

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
    State(db): State<Arc<prisma::PrismaClient>>,
) -> String {
    let mut response = format!("Project: {}\n", project);

    for (key, value) in data {
        let entry = format!("{}: {}\n", key, value);

        response.push_str(&entry)
    }

    response
}

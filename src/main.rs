use prisma::{collection, dataset, new_client};
use prisma_client_rust::BatchContainer;

use std::{collections::HashMap, future::IntoFuture, sync::Arc};
use tokio::net::TcpListener;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};

mod prisma;

#[tokio::main]
async fn main() {
    println!("Starting server...");
    let db = Arc::new(new_client().await.expect("Failed to create client"));
    println!("Database connected");

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

    let set = match db
        .dataset()
        .find_unique(dataset::UniqueWhereParam::NameEquals(project.clone()))
        .exec()
        .await
        .expect("Failed to get project")
    {
        Some(ds) => ds,
        None => db
            .dataset()
            .create_unchecked(project, vec![])
            .exec()
            .await
            .expect("Failed to create project"),
    };

    let col = db
        .collection()
        .create(dataset::UniqueWhereParam::IdEquals(set.id), vec![])
        .exec()
        .await
        .expect("Failed to create collection");

    for (key, value) in data {
        db.data_point()
            .create(
                collection::UniqueWhereParam::IdEquals(col.id),
                key.clone(),
                value.clone(),
                vec![],
            )
            .exec()
            .await
            .expect("Failed to create data point");

        let entry = format!("{}: {}\n", key, value);

        response.push_str(&entry)
    }

    response
}

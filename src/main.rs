use prisma::{
    collection::{self, WhereParam},
    dataset, new_client,
};
use prisma_client_rust::BatchContainer;

use std::{
    collections::{HashMap, HashSet},
    future::IntoFuture,
    sync::Arc,
};
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
        .route("/csv/*path", get(csv))
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

    println!("Adding data to project: {}", project);

    // Get or create a dataset
    let set: dataset::Data = db
        .dataset()
        .upsert(
            dataset::name::equals(project.clone()),
            dataset::create(project, vec![]),
            vec![],
        )
        .exec()
        .await
        .unwrap();

    // Create a collection
    let col = db
        .collection()
        .create(dataset::id::equals(set.id), vec![])
        .exec()
        .await
        .expect("Failed to create collection");

    // Create data points
    for (key, value) in data {
        db.data_point()
            .create(
                collection::id::equals(col.id),
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

async fn csv(Path(project): Path<String>, State(db): State<Arc<prisma::PrismaClient>>) -> String {
    let mut response = format!("{}\n\n", project);

    // Set of column names for formatting the output
    let mut columns: HashSet<String> = HashSet::new();

    // Data as key-value pairs
    let mut data: Vec<HashMap<String, String>> = vec![];

    // Get a projects and its collections and data points
    let project = match db
        .dataset()
        // Filter on project name
        .find_unique(dataset::name::equals(project.clone()))
        // Also fetch collections and their data points
        .with(dataset::collections::fetch(vec![]).with(collection::data_points::fetch(vec![])))
        .exec()
        .await
        .expect("Failed to get project")
    {
        Some(val) => val,
        None => return "Project not found".to_string(),
    };

    for collection in project.collections.expect("No collections found") {
        let mut map = HashMap::new();

        for point in collection.data_points.expect("No data points found") {
            columns.insert(point.key.clone());
            map.insert(point.key, point.value);
        }

        data.push(map);
    }

    // Column names
    for column in &columns {
        response.push_str(column);
        response.push(',');
    }
    response.push('\n');

    for row in data {
        for column in columns.iter() {
            // Get the value for the column or default to an empty string
            let value = row.get(column).map_or("".to_string(), |x| x.to_string());

            // push to the response and add a comma
            response.push_str(&value);
            response.push(',')
        }

        response.push('\n');
    }

    response
}

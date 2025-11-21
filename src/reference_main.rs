use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use chrono::{DateTime, Days, Timelike, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=boompje dbname=postgres",
        tokio_postgres::NoTls,
    )
    .await?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let state = AppState {
        db: Arc::new(client),
    };

    let app = Router::new()
        .route("/api/customers/{id}", get(get_customer))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Clone)]
struct AppState {
    db: Arc<Client>,
}

async fn get_customer(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Customer>, StatusCode> {
    let rows = state
        .db
        .query(
            "SELECT id, first_name, last_name FROM customers WHERE id = $1",
            &[&id],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = &rows[0];
    let user = Customer {
        id: row.get(0),
        first_name: row.get(1),
        last_name: row.get(2),
    };

    Ok(Json(user))
}

#[derive(Serialize, Deserialize)]
struct Customer {
    id: i32,
    first_name: String,
    last_name: String,
}

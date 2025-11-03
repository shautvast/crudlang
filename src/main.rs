use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::routing::any;
use axum::{Json, Router};
use crudlang::chunk::Chunk;
use crudlang::errors::Error::Platform;
use crudlang::vm::interpret_async;
use crudlang::{compile_sourcedir, map_underlying};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), crudlang::errors::Error> {
    tracing_subscriber::fmt::init();

    let registry = compile_sourcedir("source")?;

    let registry = Arc::new(registry);
    if !registry.is_empty() {
        let state = Arc::new(AppState {
            registry: registry.clone(),
        });

        let app = Router::new()
            .route("/", any(handle_any).with_state(state.clone()))
            .route("/{*path}", any(handle_any).with_state(state.clone()));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .map_err(map_underlying())?;

        println!(
            "listening on {}",
            listener.local_addr().map_err(map_underlying())?
        );

        axum::serve(listener, app).await.map_err(map_underlying())?;
        Ok(())
    } else {
        Err(Platform("No source files found".to_string()))
    }
}

#[derive(Clone)]
struct AppState {
    registry: Arc<HashMap<String, Chunk>>,
}

async fn handle_any(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Json<String>, StatusCode> {
    let method = req.method().to_string().to_ascii_lowercase();
    let uri = req.uri();

    // // todo value = Vec<String>
    // let query_params: HashMap<String, String> = uri
    //     .query()
    //     .map(|q| {
    //         url::form_urlencoded::parse(q.as_bytes())
    //             .into_owned()
    //             .collect()
    //     })
    //     .unwrap_or_default();
    let component = format!("{}/web.{}", &uri.path()[1..], method);
    Ok(Json(
        interpret_async(&state.registry, &component, req)
            .await
            .unwrap()
            .to_string(),
    ))
}

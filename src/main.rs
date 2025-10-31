use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use crudlang::ast_compiler;
use crudlang::bytecode_compiler::compile;
use crudlang::chunk::Chunk;
use crudlang::scanner::scan;
use crudlang::vm::{interpret, interpret_async};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::sync::Arc;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut paths = HashMap::new();
    let mut registry = HashMap::new();
    for entry in WalkDir::new("source").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.ends_with("web.crud") {
            print!("compiling {:?}: ", path);
            let source = fs::read_to_string(path)?;
            let tokens = scan(&source)?;
            match ast_compiler::compile(tokens) {
                Ok(statements) => {
                    let path = path
                        .strip_prefix("source")?
                        .to_str()
                        .unwrap()
                        .replace(".crud", "");
                    let chunk = compile(Some(&path), &statements, &mut registry)?;
                    paths.insert(path, chunk);
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            println!();
        }
    }

    let registry = Arc::new(registry);
    if !paths.is_empty() {
        let mut app = Router::new();
        for (path, code) in paths.iter() {
            let state = Arc::new(AppState {
                name: format!("{}.get", path),
                registry: registry.clone(),
            });
            println!("adding {}", path);
            app = app.route(
                &format!("/{}", path.replace("/web", "")),
                get(handle_get).with_state(state.clone()),
            );
        }
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        println!("listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await?;
    }
    Ok(())
}

#[derive(Clone)]
struct AppState {
    name: String,
    registry: Arc<HashMap<String, Chunk>>,
}

async fn handle_get(State(state): State<Arc<AppState>>) -> Result<Json<String>, StatusCode> {
    Ok(Json(
        interpret_async(&state.registry, &state.name)
            .await
            .unwrap()
            .to_string(),
    ))
}


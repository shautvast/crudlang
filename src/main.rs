use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use crudlang::ast_compiler;
use crudlang::bytecode_compiler::compile;
use crudlang::chunk::Chunk;
use crudlang::scanner::scan;
use crudlang::vm::interpret;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut paths = HashMap::new();
    for entry in WalkDir::new("source").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.ends_with("web.crud") {
            print!("compiling {:?}: ", path);
            let source = fs::read_to_string(path)?;
            let tokens = scan(&source);
            match ast_compiler::compile(tokens) {
                Ok(statements) => {
                    let chunk = compile(&statements)?;
                    let path = path.strip_prefix("source")?.to_str().unwrap();
                    let path = path.replace("/web.crud", "");
                    paths.insert(format!("/{}", path), chunk);
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            println!();
        }
    }
    if !paths.is_empty() {
        let mut app = Router::new();
        for (path, code) in paths.iter() {
            let code = code.functions.get("get").unwrap();
            let state = Arc::new(AppState { code: code.clone() });
            println!("adding {}", path);
            app = app.route(path, get(handle_get).with_state(state.clone()));
            // .with_state(state);
        }
        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        println!("listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await?;
    }
    Ok(())
}

#[derive(Clone)]
struct AppState {
    code: Chunk,
}

async fn handle_get(State(state): State<Arc<AppState>>) -> Result<Json<String>, StatusCode> {
    Ok(Json(interpret(&state.code).await.unwrap().to_string()))
}

//

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compile() -> anyhow::Result<()> {
        let tokens = scan(
            r#"
fn hello(name: string) -> string:
    "Hello "+name
hello("sander")"#,
        );

        match ast_compiler::compile(tokens) {
            Ok(statements) => {
                println!("{:?}", statements);
                let chunk = compile(&statements)?;
                chunk.disassemble();
                println!("{}", interpret(&chunk).await?);
            }
            Err(e) => {
                println!("{}", e)
            }
        }
        Ok(())
    }
}

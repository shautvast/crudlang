use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::routing::any;
use axum::{Json, Router};
use clap::Parser;
use crudlang::chunk::Chunk;
use crudlang::errors::CrudLangError;
use crudlang::errors::CrudLangError::Platform;
use crudlang::vm::interpret_async;
use crudlang::{compile_sourcedir, map_underlying};
use notify::Watcher;
use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use log::{debug, info};

/// A simple CLI tool to greet users
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    repl: bool,

    #[arg(short, long)]
    source: Option<String>,

    #[arg(short, long)]
    watch: bool,
}

#[tokio::main]
async fn main() -> Result<(), CrudLangError> {
    println!("-- Crudlang --");
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let source = args.source.unwrap_or("./source".to_string());
    let registry = compile_sourcedir(&source)?;
    let empty = registry.is_empty();

    let swap = Arc::new(ArcSwap::from(Arc::new(registry)));
    if !empty {
        if args.watch {
            crudlang::file_watch::start_watch_daemon(&source, swap.clone());
        }
        println!("-- Compilation successful --");
        let state =AppState {
            registry: swap.clone(),
        };
        let app = Router::new()
            .route("/", any(handle_any).with_state(state.clone()))
            .route("/{*path}", any(handle_any).with_state(state.clone()));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .map_err(map_underlying())?;

        println!(
            "-- Listening on {} --\n",
            listener.local_addr().map_err(map_underlying())?
        );

        if args.repl {
            std::thread::spawn(move || crudlang::repl::start(swap.clone()).unwrap());
        }

        axum::serve(listener, app).await.map_err(map_underlying())?;
    } else {
        println!("No source files found or compilation error");
        if args.repl {
            crudlang::repl::start(swap.clone())?;
        }
    }
    Ok(())
}

#[derive(Clone)]
struct AppState {
    registry: Arc<ArcSwap<HashMap<String, Chunk>>>,
}

async fn handle_any(
    State(state): State<AppState>,
    req: Request,
) -> Result<Json<String>, StatusCode> {
    let method = req.method().to_string().to_ascii_lowercase();
    let uri = req.uri();

    // // todo value = Vec<String>
    let query_params: HashMap<String, String> = uri
        .query()
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();
    let component = format!("{}/web", &uri.path());
    let function_qname = format!("{}/{}", component, method);

    let mut headers = HashMap::new();
    for (k, v) in req.headers().iter() {
        headers.insert(k.to_string(), v.to_str().unwrap().to_string());
    }
    let path = &req.uri().to_string();
    info!("invoked {:?} => {}",req, function_qname);
    match interpret_async(
        state.registry.load(),
        &function_qname,
        path,
        query_params,
        headers,
    )
    .await
    {
        Ok(value) => Ok(Json(value.to_string())),
        Err(_) => {
            // url checks out but function for method not found
            if state.registry.load().get(&format!("{}.main", component)).is_some() {
                Err(StatusCode::METHOD_NOT_ALLOWED)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
    }
}

use crate::chunk::Chunk;
use crate::compile_sourcedir;
use log4rs::append::Append;
use notify::{RecursiveMode, Watcher};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, SystemTime};
use arc_swap::ArcSwap;

const ONE_SEC: Duration = Duration::from_secs(1);

pub fn start_watch_daemon(source: &str, registry: Arc<ArcSwap<HashMap<String, Chunk>>>) {
    let source = source.to_string();
    let s = source.to_string();
    let (tx, rx) = channel();
    thread::spawn(move || {
        println!("-- File watch started --");
        let path = Path::new(&source);
        if !path.exists() {
            panic!("source directory {} does not exist", &source);
        }
        let mut watcher = notify::recommended_watcher(tx).expect("Failed to create watcher");
        watcher
            .watch(path, RecursiveMode::Recursive)
            .expect("Failed to watch");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });
    thread::spawn(move || {
        let mut file_changed = false;
        loop {
            let start = SystemTime::now();
            loop {
                thread::sleep(Duration::from_millis(50));
                if let Ok(_) = rx.recv() {
                    file_changed = true;
                }
                if file_changed && SystemTime::now().duration_since(start).unwrap() > ONE_SEC {
                    break;
                }
            }
            println!("refresh"); // TODO implement refresh source
            let new_registry = Arc::new(compile_sourcedir(&s).unwrap());

            registry.store(new_registry.clone());
            file_changed = false;
        }
    });
}

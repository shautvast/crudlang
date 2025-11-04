use crate::chunk::Chunk;
use crate::errors::CrudLangError;
use crate::map_underlying;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::sync::Arc;

pub fn start(registry: Arc<HashMap<String, Chunk>>) -> Result<(), CrudLangError> {
    println!("REPL started -- Type ctrl-c to exit (both the repl and the server)");
    println!(":h for help");
    loop {
        print!(">");
        io::stdout().flush().map_err(map_underlying())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(map_underlying())?;
        let input = input.trim();
        match input {
            ":h" => help(),
            ":le" => list_endpoints(registry.clone()),
            ":lf" => list_functions(registry.clone()),
            _ => {}
        }
        // println!("[{}]",input);
        // if input == ":q" {
        //     break;
        // }
    }
    // println!("-- Crudlang -- REPL exited");
    Ok(())
}

fn list_endpoints(registry: Arc<HashMap<String, Chunk>>) {
    registry
        .iter()
        .filter(|(k, _)| k.contains("get"))
        .for_each(|(k, _)| {
            println!("{}", k);//number
        });
}

fn list_functions(registry: Arc<HashMap<String, Chunk>>) {
    registry
        .iter()
        .for_each(|(k, _)| {
            println!("{}", k);//number
        });
}

fn help() {
    println!(":le\t lists all registered endpoints");
    println!(":lf\t lists all registered functions");
}

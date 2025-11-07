use crate::chunk::Chunk;
use crate::errors::CrudLangError;
use crate::vm::interpret;
use crate::{map_underlying, recompile};
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;

pub fn start(registry: Arc<ArcSwap<HashMap<String, Chunk>>>) -> Result<(), CrudLangError> {
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
            ":le" => list_endpoints(registry.load().clone()),
            ":lf" => list_functions(registry.load().clone()),
            _ => {
                let registry_copy = registry.load().clone();
                let mut registry_copy = registry_copy.deref().clone();
                match recompile(&input, &mut registry_copy){
                    Ok(_)=> {
                        registry.store(Arc::new(registry_copy));

                        let result = match interpret(registry.load(), "main") {
                            Ok(value) => value.to_string(),
                            Err(e) => e.to_string(),
                        };
                        println!("{}", result);
                    },
                    Err(e)  => {
                        println!("{}", e);
                    }
                }

            }
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
            println!("{}", k); //number
        });
}

fn list_functions(registry: Arc<HashMap<String, Chunk>>) {
    registry.iter().for_each(|(k, _)| {
        println!("{}", k); //number
    });
}

fn help() {
    println!(":le\t lists all registered endpoints");
    println!(":lf\t lists all registered functions");
}

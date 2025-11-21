use crate::compiler::asm_pass::AsmChunk;
use crate::compiler::scan_pass::scan;
use crate::compiler::{asm_pass, ast_pass, map_underlying};
use crate::errors::TipiLangError;
use crate::symbol_builder;
use crate::vm::Vm;
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;

pub fn start(registry: Arc<ArcSwap<HashMap<String, AsmChunk>>>) -> Result<(), TipiLangError> {
    println!("REPL started -- Type ctrl-c to exit (both the repl and the server)");
    println!(":h for help");
    let mut symbol_table = HashMap::new();
    let mut vm = Vm::new(&registry.load());
    let mut asm_pass = asm_pass::AsmPass::new("main");
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

                let tokens = scan(input)?;

                let ast = match ast_pass::compile(None, tokens, &mut symbol_table) {
                    Ok(ast) => ast,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
                symbol_builder::build("", &ast, &mut symbol_table);

                match asm_pass.compile(&ast, &symbol_table, &mut registry_copy, "") {
                    Ok(chunk) => {
                        registry_copy.insert("main".to_string(), chunk);
                        registry.store(Arc::new(registry_copy));

                        let result = match vm.run("main", registry.load().get("main").unwrap()) {
                            Ok(value) => value.to_string(),
                            Err(e) => e.to_string(),
                        };
                        println!("{}", result);
                    }
                    Err(e) => {
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
}

fn list_endpoints(registry: Arc<HashMap<String, AsmChunk>>) {
    registry
        .iter()
        .filter(|(k, _)| k.contains("get"))
        .for_each(|(k, _)| {
            println!("{}", k); //number
        });
}

fn list_functions(registry: Arc<HashMap<String, AsmChunk>>) {
    registry.iter().for_each(|(k, _)| {
        println!("{}", k); //number
    });
}

fn help() {
    println!(":le\t lists all registered endpoints");
    println!(":lf\t lists all registered functions");
}

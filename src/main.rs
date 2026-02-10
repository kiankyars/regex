mod ast;
mod compiler;
mod parser;
mod vm;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: regex-engine <pattern> <input>");
        std::process::exit(1);
    }

    let pattern = &args[1];
    let input = &args[2];

    // Parse
    let mut p = parser::Parser::new(pattern);
    let ast = match p.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("ERROR:{}", e);
            return;
        }
    };

    let n_groups = p.group_count();

    // Compile
    let program = compiler::compile(&ast, n_groups);

    // Execute
    match vm::search(&program, input) {
        Some(result) => {
            let matched: String = input.chars().skip(result.start).take(result.end - result.start).collect();
            println!("MATCH:{}", matched);
            // Print capturing groups
            for i in 1..=n_groups {
                let start_slot = i * 2;
                let end_slot = i * 2 + 1;
                match (result.captures.get(start_slot).copied().flatten(),
                       result.captures.get(end_slot).copied().flatten()) {
                    (Some(s), Some(e)) => {
                        let group_text: String = input.chars().skip(s).take(e - s).collect();
                        println!("GROUP {}:{}", i, group_text);
                    }
                    _ => {
                        println!("GROUP {}:", i);
                    }
                }
            }
        }
        None => {
            println!("NO_MATCH");
        }
    }
}

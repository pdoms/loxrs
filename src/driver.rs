use std::io::stdout;
use std::{io::Write, path::Path};

use crate::resolver::Resolver;
use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

pub fn run(input_file: &str) -> Result<(), ()> {
    let path = Path::new(input_file);
    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[ERROR] error loading file: {}: {err}", input_file);
            return Err(());
        }
    };
    let mut scanner = Scanner::new(&data);
    match scanner.parse() {
        Ok(_) => {}
        Err(errors) => {
            println!("[ERROR] scanner encountered the following errors");
            for error in errors {
                print!("{}{}", input_file, error);
            }
        }
    }
    println!("Parsed {} tokens.", scanner.tokens.len());

    //TODO do we really need it as ref? Could also just be passed on
    let mut parser = Parser::new(&scanner.tokens);
    let stmts = match parser.parse() {
        Ok(res) => res,
        Err(errors) => {
            println!("[ERROR] parser encountered the following errors");
            for error in errors {
                print!("{}{}", input_file, error);
            }
            vec![]
        }
    };

    let mut resolver = Resolver::new();
    for stmt in &stmts {
        if let Err(err) = resolver.resolve_stmt(stmt) {
            println!("[ERROR] {err}",);
            return Err(());
        }
    }

    let mut stdout = stdout();
    println!("Parsed {} statements", stmts.len());
    let mut interpreter = Interpreter::new(&mut stdout);
    interpreter.resolve(resolver.locals);
    println!("================ Program StdOut ================");
    match interpreter.interpret(&stmts) {
        Ok(_) => match stdout.flush() {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("[ERROR] flushing stdout: {err}");
                Err(())
            }
        },
        Err(runtime) => {
            eprintln!("[ERROR] {}", runtime);
            Err(())
        }
    }
}

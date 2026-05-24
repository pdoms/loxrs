use std::io::stdout;
use std::{io::Write, path::Path};

use crate::resolver::Resolver;
use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

#[allow(clippy::result_unit_err)]
pub fn run(input_file: &str, verbose: bool) -> Result<(), ()> {
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
            eprintln!("[ERROR] scanner encountered the following errors");
            for error in errors {
                eprint!("{}{}", input_file, error);
            }
            return Err(());
        }
    }
    if verbose {
        println!("[INFO] Parsed {} tokens.", scanner.tokens.len());
    }

    let mut parser = Parser::new(&scanner.tokens);
    let stmts = match parser.parse() {
        Ok(res) => res,
        Err(errors) => {
            eprintln!("[ERROR] parser encountered the following errors");
            for error in errors {
                eprint!("{}{}", input_file, error);
            }
            return Err(());
        }
    };

    let mut resolver = Resolver::default();
    for stmt in &stmts {
        if let Err(err) = resolver.resolve_stmt(stmt) {
            eprintln!("[ERROR] {err}",);
            return Err(());
        }
    }

    let mut stdout = stdout();
    if verbose {
        println!("[INFO] Parsed {} statements", stmts.len());
    }
    let mut interpreter = Interpreter::new(&mut stdout);
    interpreter.resolve(resolver.locals);
    if verbose {
        println!("================ Program StdOut ================");
    }
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

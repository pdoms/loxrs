use std::path::Path;

use crate::{interpreter::eval, parser::Parser, scanner::Scanner};

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
    let expr = parser.parse().unwrap();
    println!("AST: {}", expr);
    match eval(&expr) {
        Ok(v) => println!("{v}"),
        Err(err) => eprintln!("{err}"),
    }

    Ok(())
}

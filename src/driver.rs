use std::path::Path;

use crate::scanner::Scanner;

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

    Ok(())
}

mod driver;
mod environment;
mod errors;
mod interpreter;
mod native;
mod nodes;
mod parser;
mod resolver;
mod scanner;
mod token;

fn usage(err: &str, program: &str) {
    eprintln!("Usage: {} [--verbose/-v] <FILE> ", program);
    eprintln!("  FILE ................. the path to the input file");
    eprintln!("  --verbose/-v ......... print debug information");
    if !err.is_empty() {
        eprintln!("[ERROR] {err}");
    }
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap(); // will always be there
    let mut verbose = false;
    let mut in_file = None;
    for arg in args {
        match arg.as_str() {
            "--verbose" | "-v" => verbose = true,
            _ => in_file = Some(arg),
        }
    }
    match in_file {
        Some(f) => {
            if driver::run(&f, verbose).is_err() {
                std::process::exit(1);
            }
        }
        None => {
            usage("no input file was provided", &program);
            std::process::exit(1);
        }
    }
}

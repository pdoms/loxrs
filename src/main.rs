mod driver;
mod errors;
mod scanner;
mod token;

fn usage(err: &str, program: &str) {
    println!("{} <FILE>", program);
    println!("FILE ............ the path to the input file");
    if !err.is_empty() {
        eprintln!("[ERROR] {err}");
    }
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap(); // will always be there
    // for now we only expect 1 argument and if it is not there, we bail
    match args.next() {
        Some(in_file) => {
            if let Err(()) = driver::run(&in_file) {
                std::process::exit(1);
            }
        }
        None => {
            usage("no input file was provided", &program);
            std::process::exit(1);
        }
    }
}

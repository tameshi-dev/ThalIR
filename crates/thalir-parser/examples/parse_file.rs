use std::env;
use std::fs;
use thalir_parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.thalir>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    match fs::read_to_string(filename) {
        Ok(content) => match parse(&content) {
            Ok(_) => {
                println!(" {} parsed successfully", filename);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!(" Parse error in {}: {}", filename, e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!(" Failed to read {}: {}", filename, e);
            std::process::exit(1);
        }
    }
}

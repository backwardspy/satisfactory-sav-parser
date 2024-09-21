use std::{fs::File, io::BufReader};

fn main() {
    let Some(ref save_path) = std::env::args().nth(1) else {
        let cmd = std::env::args().next().unwrap();
        eprintln!("Usage: {cmd} <path>");
        std::process::exit(1);
    };

    let mut parser = {
        let Ok(file) = File::open(save_path) else {
            eprintln!("Failed to open file: {save_path}");
            std::process::exit(1);
        };
        satisfactory_sav_parser::Parser::new(BufReader::new(file))
    };

    let header = parser.read_header().expect("can read header");

    println!("{header:#?}");
}

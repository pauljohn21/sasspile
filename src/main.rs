use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sasspile <file.scss>");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    match sasspile::compile_file(&path) {
        Ok(css) => print!("{}", css),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

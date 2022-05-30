use std::env;
use std::process;

mod lib;
use lib::Target;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let target: Target = Target::new(&args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    if let Err(e) = lib::run(&target) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

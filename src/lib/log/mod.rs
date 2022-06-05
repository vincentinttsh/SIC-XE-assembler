pub fn print(msg: &str, verbose: bool) {
    if verbose {
        print!("{}", msg);
    }
}

pub fn println(msg: &str, verbose: bool) {
    if verbose {
        println!("{}", msg);
    }
}
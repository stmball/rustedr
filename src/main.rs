use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();

    match rustedr::run(&args[1]) {
        Ok(_) => println!("Completed"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

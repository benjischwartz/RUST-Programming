use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let arg = match args.get(1) {
        Some(s) => s,
        None => "",
    };
    println!("arg = {}", arg);
    println!("upp = {}", arg.to_uppercase());
}
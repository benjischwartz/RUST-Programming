use std::io;
use std::io::Write;

fn main() {
    let mut input = String::new();
    print!("Hello! What is your name: ");
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).expect("Error reading from STDIN");
    input.pop();
    if input.len() == 0 {
        println!("You didn't enter a name!");
    } else {
        println!("Hello, {input}! Nice to meet you :)");
    }
}

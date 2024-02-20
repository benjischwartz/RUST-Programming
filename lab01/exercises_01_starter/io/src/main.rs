use std::io;
use std::io::Write;

fn main() {
    let mut input = String::new();
    print!("What is your name? ");
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).expect("Error reading from STDIN");
    //input.pop();
    if input.len() == 1 {
        println!("No name entered :(, goodbye.");
    } else {
        println!("Hello, {input}, nice to meet you!");
    }
}

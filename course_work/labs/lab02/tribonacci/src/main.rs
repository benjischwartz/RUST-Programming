use std::env;
use std::num::ParseIntError;

struct TribonacciError(String);

fn main() {
    let args: Vec<String> = env::args().collect();
    let error_message = String::from("Please enter a valid size");

    let size = match args.get(1) {
        Some(s) => s.parse::<usize>(),
        None => Ok(10),
    };

    if let Err(e) = compute_tribonacci(size, error_message) {
        println!("Error: {}", e.0)
    }
}

/// Computes the tribonacci sequence of a given size
/// Prints the sequence, and its sum
fn compute_tribonacci(
    size: Result<usize, ParseIntError>,
    // The error message your function should return
    // inside the `TribonacciError` struct
    error_msg: String,
) -> Result<(), TribonacciError> {
    let mut v = vec![1, 1, 1];
    match size {
        Ok(size) => {
            let mut sum = size as u128;
            if size > 3 {
                sum = 3;
                let mut i = 3;
                while i < size {
                    v.push(v[i - 3] + v[i - 2] + v[i - 1]);
                    sum = sum + v[i];
                    i = i + 1;
                }
            }
            println!("Values: {v:?}\n");
            println!("Sum: {sum}");
            Ok(())
        }
        Err(message) => {
            Err(TribonacciError(error_msg))
        }
    }
}
use std::io::{stdin, BufRead};
use std::env;
use std::char;

const DEFAULT_SHIFT: i32 = 5;

fn main() {
    let shift_by: i32 = env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(DEFAULT_SHIFT);
    
    for line in stdin().lock().lines() {
        match line {
            Ok(mut line) => {
                let res = shift(shift_by, &line);
                println!("Shifted ascii by {shift_by} is: {res}");
            }
            Err(_) => {
                println!("no input line");
            }
        }
    }
}


fn shift(shift: i32, line: &String) -> String {
    let mut res = String::new();
    res += &line
        .chars()
        .map(|c| shift_char(c, shift))
        .collect::<String>();
    res
}

fn shift_char(c: char, shift: i32) -> char {
    if c >= 'a' && c <= 'z'
    {
        return (((c as i32 - 'a' as i32 + shift) % 26) as u8 + 'a' as u8) as char
    }
    else if c >= 'A' && c <= 'Z'
    {
        return (((c as i32 - 'A' as i32 + shift) % 26) as u8 + 'A' as u8) as char
    }
    c
}

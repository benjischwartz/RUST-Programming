use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::vec::IntoIter;
use itertools::{Chunk, Itertools};
mod test;
fn main() {
    // take number from commandline arg
    // number is guaranteed to be five digits
    let input_number = std::env::args().nth(1).unwrap().parse::<u32>().unwrap();
    if !(10000..=99999).contains(&input_number) {
        panic!("Number must be five digits");
    }

    let operators = vec!['+', '-', '*', '/'];

    // let's get a massive iterator,
    // over every arrangement of
    // digits and every arrangement of operators
    let digits_operators: Vec<(Vec<i32>, Vec<char>)> = std::env::args()
        .nth(1)
        .unwrap()
        .chars()
        .map(|x| x.to_digit(10).unwrap() as i32)
        .permutations(5)
        .into_iter()
        .cartesian_product(operators.into_iter().permutations(4).into_iter())
        .collect();

    let length = digits_operators.len();
    println!("There are {length} potential combinations",);

    // you only need to change code from here onwards
    // first, split up the digits_operators into 6 vecs
    // using the chunks method
    let num_threads = 16usize;
    let mut chunked: Vec<Vec<(Vec<i32>, Vec<char>)>> = Vec::new();
    let mut calculation_count: Vec<Receiver<usize>> = Vec::new();

    for chunk in &digits_operators.into_iter().chunks(length/num_threads) {
        chunked.push(chunk.collect_vec())
    }


    for chunk in chunked {
        // Create channel
        let (tx, rx) = channel();
        calculation_count.push(rx);
        thread::scope(|s| {
            s.spawn(|| {
                let mut num_calculations = 0usize;
                for (digits, operators) in chunk {
                    match calculate(digits, operators) {
                        Ok(true) => { num_calculations += 1; }
                        _ => {},
                    };
                }
                tx.send(num_calculations);
            });
        });
    }

    let mut total = 0usize;
    for (i, thread) in calculation_count.into_iter().enumerate() {
        let num_calculations = thread.recv().unwrap();
        total += num_calculations;
        println!{"Thread {i} found {num_calculations} combinations"};
    }

    println!("Total: {total}");
}

// DO NOT MODIFY
fn calculate(digits: Vec<i32>, operators: Vec<char>) -> Result<bool, ()> {
    let num1 = digits[0];
    let num2 = digits[1];
    let num3 = digits[2];
    let num4 = digits[3];
    let num5 = digits[4];

    let op1 = operators[0];
    let op2 = operators[1];
    let op3 = operators[2];
    let op4 = operators[3];

    let result = operate(num1, num2, op1)?;
    let result = operate(result, num3, op2)?;
    let result = operate(result, num4, op3)?;
    let result = operate(result, num5, op4)?;
    if result == 10 {
        println!(
            "{} {} {} {} {} {} {} {} {} = 10",
            num1, op1, num2, op2, num3, op3, num4, op4, num5
        );
        return Ok(true)
    }

    Ok(false)
}

// DO NOT MODIFY
fn operate(num1: i32, num2: i32, op: char) -> Result<i32, ()> {
    match op {
        '+' => Ok(num1 + num2),
        '-' => Ok(num1 - num2),
        '*' => Ok(num1 * num2),
        '/' => num1.checked_div(num2).ok_or(()),
        _ => panic!("Invalid operation"),
    }
}

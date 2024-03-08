//! Encrypts a message using the Caesar cypher, with an optional shift value.
//! Provides the function caesar_shift.

/// Default shift is 5 if none provided.
pub const DEFAULT_SHIFT: i32 = 5;
/// ASCII value of 'A'
pub const UPPERCASE_A: i32 = 65;
/// ASCII value of 'a'
pub const LOWERCASE_A: i32 = 97;
/// Number of characters in the english alphabet
pub const ALPHABET_SIZE: i32 = 26;


/// Reads in a single command-line argument, which is the number
/// of positions to shift each letter (i32).
/// If there is no command-line argument, or if the argument is
/// not a number, the program should use a default shift of 5.
/// # Examples
/// ```
/// # use doctor_who::caesar_shift;
/// let vec = vec![String::from("hello!")];
/// assert_eq!(caesar_shift(None, vec), vec!["mjqqt!"]);
/// let vec = vec![String::from("world")];
/// assert_eq!(caesar_shift(Some(1), vec), vec!["xpsme"]);
/// ```
pub fn caesar_shift(shift_by: Option<i32>, lines: Vec<String>) -> Vec<String> {
    let shift_number = shift_by.unwrap_or(DEFAULT_SHIFT);
    
    // no idea what this is doing? Ask the forums and/or 
    // look back at the functional programming lectures!
    lines
        .iter()
        .map(|line| shift(shift_number, line.to_string()))
        .collect()
}
fn shift(shift_by: i32, line: String) -> String {
    let mut result: Vec<char> = Vec::new();

    // turn shift_by into a positive number between 0 and 25
    let shift_by = shift_by % ALPHABET_SIZE + ALPHABET_SIZE;

    line.chars().for_each(|c| {
        let ascii = c as i32;

        if ('A'..='Z').contains(&c) {
            result.push(to_ascii(
                abs_modulo((ascii - UPPERCASE_A) + shift_by, ALPHABET_SIZE) + UPPERCASE_A,
            ));
        } else if ('a'..='z').contains(&c) {
            result.push(to_ascii(
                abs_modulo((ascii - LOWERCASE_A) + shift_by, ALPHABET_SIZE) + LOWERCASE_A,
            ));
        } else {
            result.push(c)
        }
    });

    result.iter().collect()
}

fn abs_modulo(a: i32, b: i32) -> i32 {
    (a % b).abs()
}

fn to_ascii(i: i32) -> char {
    char::from_u32(i as u32).unwrap()
}

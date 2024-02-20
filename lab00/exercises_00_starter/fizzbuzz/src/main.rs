fn main() {
    for i in 1..=100 {
        let three_remainder = i % 3;
        let five_remainder = i % 5;
        if three_remainder == 0 && five_remainder == 0{
            println!("FizzBuzz");
        } else if three_remainder == 0 {
            println!("Fizz");
        } else if five_remainder == 0 {
            println!("Buzz");
        } else {
            println!("{i}");
        }
    }
}

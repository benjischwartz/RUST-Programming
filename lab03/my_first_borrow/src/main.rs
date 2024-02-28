fn main() {
    let s = String::from("hello, ");
    let t = String::from("world!");

    print_strings(&s, &t);

    println!("I want to use these strings! {}{}", s, t);
}

fn print_strings(s: &String, t: &String) {
    println!("inside print_strings: {}{}", s, t);
}

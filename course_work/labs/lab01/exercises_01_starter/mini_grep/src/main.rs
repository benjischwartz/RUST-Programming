fn main() {
    loop{
        let pattern_string = std::env::args()
            .nth(1)
            .expect("missing required command-line argument: <pattern>");

        let pattern = &pattern_string;
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
        if line.is_empty() {
            break;
        }
        if line.find(&pattern_string) != None {
            print!("{}", line);
        }
    }
}

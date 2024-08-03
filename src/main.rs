use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else if pattern == "\\d" {
        return input_line.contains(|c: char| c.is_digit(10));
    } else if pattern == "\\s" {
        return input_line.contains(|c: char| c.is_whitespace());
    } else if pattern == "\\w" {
        return input_line.contains(|c: char| c.is_alphanumeric());
    } else if pattern.starts_with("[") && pattern.ends_with("]") {
        let mut pattern_chars = pattern.chars();
        pattern_chars.next();
        pattern_chars.next_back();
        let mut pattern_chars: Vec<char> = pattern_chars.collect();
        if pattern_chars.first() == Some(&'^') {
            pattern_chars.remove(0);
            return input_line.contains(|c: char| !pattern_chars.contains(&c));
        } else {
            return input_line.contains(|c: char| pattern_chars.contains(&c));
        }
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

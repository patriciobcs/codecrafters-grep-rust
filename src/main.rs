use std::env;
use std::io;
use std::process;

fn match_next(next_char: char, conditions: &Vec<&str>) -> bool {
    let mut is_matching = false;

        for condition in conditions {
            match condition {
                &"\\d" if next_char.is_digit(10) => {
                        is_matching = true;
                },
                &"\\s" => {
                    if next_char.is_whitespace() {
                        is_matching = true;
                    }
                },
                &"\\w" => {
                    if next_char.is_alphanumeric() {
                        is_matching = true;
                    }
                },
                x => {
                    if next_char.to_string() == *x {
                        is_matching = true;
                    }
                }
            }
        }

    is_matching
}

fn match_pattern(input_line: &mut Vec<char>, pattern: &mut Vec<char>, must_match: bool) -> bool {
    // If no more input line, it's not matching (pattern is not empty)
    if input_line.len() == 0 {
        return false
    }

    let (is_matching, drain_ends) = match pattern.first() {
        Some(&'^') if !must_match => {
            return match_pattern(input_line, &mut pattern[1..].to_vec(), true)
        },
        Some(&'\\') => {
            (match_next(input_line[0], &vec![&pattern[0..=1].iter().collect::<String>()]), 1)
        },
        Some(&'[') => {
            let mut conditions: Vec<String> = vec![];

            let mut i = 0;
            
            let mut is_negated = false;

            while i < pattern.len() {
                i+=1;
                if pattern[i] == ']' {
                    break;
                } else if pattern[i] == '^' {
                    is_negated = true;
                }

                let condition = if pattern[i] == '\\' {
                    i+=1;
                    pattern[i-1..=i].iter().collect::<String>()
                } else {
                    pattern[i].to_string()
                };
                
                conditions.push(condition);
            }

            let conditions: Vec<&str> = conditions.iter().map(|x| x.as_str()).collect();

            let is_matching = match_next(input_line[0], &conditions);

            (is_negated != is_matching, i)
        },
        _ => {
            (match_next(input_line[0], &vec![&pattern.first().unwrap().to_string()]), 0)
        }
    };
    
    if is_matching {
        // Local match, remove matched pattern
        pattern.drain(0..=drain_ends);

        // If no more pattern, it's a global match
        if pattern.len() == 0 {
            return true
        }
    // This match was mandatory but it didn't happen
    } else if must_match {
        return false
    }
    
    // Remove first character from input line
    input_line.remove(0);

    // Recursively call match_pattern with the rest of the input line
    match_pattern(input_line, pattern, must_match || is_matching)
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let mut pattern: Vec<char> = env::args().nth(2).unwrap().chars().collect();

    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    let mut input_line: Vec<char> = input_line.chars().collect();

    if match_pattern(&mut input_line, &mut pattern, false) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

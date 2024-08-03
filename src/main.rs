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

fn match_pattern(input_line: &mut Vec<char>, pattern: &mut Vec<char>, next: bool) -> bool {
    let (is_matching, drain_ends) = match pattern.first() {
        Some(&'^') => {
            (match_pattern(input_line, pattern, true), 0)
        },
        Some(&'\\') => {
            (match_next(input_line[0], &vec![&pattern[0..=1].iter().collect::<String>()]), 1)
        },
        Some(&'[') => {
            let mut conditions: Vec<String> = vec![];

            let mut i = 0;
            while i < pattern.len() {
                i+=1;
                if pattern[i] == ']' {
                    break;
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


            (match_next(input_line[0], &conditions), i)
        },
        _ => {
            (match_next(input_line[0], &vec![&pattern.first().unwrap().to_string()]), 0)
        }
    };

    if is_matching {
        pattern.drain(0..=drain_ends);
    }

    if is_matching {
        input_line.remove(0);
        if input_line.len() == 0 || pattern.len() == 0 {
            return true
        }
        match_pattern(input_line, pattern, true)
    } else {
        if next {
            false
        } else {
            input_line.remove(0);
            if input_line.len() == 0 {
                return true
            }
            match_pattern(input_line, pattern, false)
        }
    }
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

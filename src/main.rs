use std::env;
use std::io;
use std::process;

fn match_next(next_char: char, conditions: &Vec<String>) -> bool {
    let mut is_matching = false;

    for condition in conditions {
        match condition.as_str() {
            "\\d" if next_char.is_digit(10) => {
                is_matching = true;
            }
            "\\s" => {
                if next_char.is_whitespace() {
                    is_matching = true;
                }
            }
            "\\w" => {
                if next_char.is_alphanumeric() {
                    is_matching = true;
                }
            }
            "." => {
                is_matching = true;
            }
            x => {
                if next_char.to_string() == *x {
                    is_matching = true;
                }
            }
        }
    }

    is_matching
}

fn match_pattern(
    input_line: &mut Vec<char>,
    pattern: &mut Vec<char>,
    must_match: bool,
    backreferences: &mut Vec<Vec<char>>,
    char_after_group: Option<&char>,
) -> bool {
    // If no more input line, it's not matching (pattern is not empty)
    if input_line.len() == 0 {
        if pattern.first() == Some(&'$') {
            return true;
        } else {
            return false;
        }
    }

    let mut input_line_leftover = None;

    // Pattern start conditions
    let (conditions, mut pattern_matched_last_index, is_negated) = match pattern.first() {
        Some(&'^') if !must_match => {
            return match_pattern(input_line, &mut pattern[1..].to_vec(), true, backreferences, None)
        }
        Some(&'\\') => {
            if let Some(index) = pattern.get(1) {
                if index.is_digit(10) {
                    let backreference_index = index.to_digit(10).unwrap() as usize;
                    if let Some(backreference) =
                    backreferences.get(backreference_index.saturating_sub(1))
                    {
                        let pattern_leftover = if pattern.len() > 2 {
                            pattern[2..].to_vec()
                        } else {
                            vec![]
                        };

                        let updated_pattern =
                            vec![backreference.clone(), pattern_leftover].concat();

                        return match_pattern(
                            input_line,
                            &mut updated_pattern.to_vec(),
                            true,
                            backreferences,
                            char_after_group,
                        );
                    }
                }
            }

            let conditions = vec![pattern[0..=1].iter().collect::<String>()];

            (conditions, 1usize, false)
        }
        Some(&'[') => {
            let mut conditions: Vec<String> = vec![];
            let mut i: usize = 0;
            let mut is_negated = false;

            while i < pattern.len() {
                i += 1;
                if pattern[i] == ']' {
                    break;
                } else if pattern[i] == '^' {
                    is_negated = true;
                    continue;
                }

                let condition = if pattern[i] == '\\' {
                    i += 1;
                    pattern[i - 1..=i].iter().collect::<String>()
                } else {
                    pattern[i].to_string()
                };

                conditions.push(condition);
            }

            (conditions, i, is_negated)
        }
        Some(&'(') => {
            let mut i: usize = 1;
            let mut current_pattern: Vec<char> = vec![];
            let mut matched_line = None;
            let mut internal_parentheses_starts = 0;
            let mut internal_parentheses_ends = 0;

            while i < pattern.len() {
                if pattern[i] == '(' {
                    internal_parentheses_starts += 1;
                }
                
                if internal_parentheses_starts == internal_parentheses_ends && (pattern[i] == ')' || pattern[i] == '|') {
                    if internal_parentheses_starts > 0 {
                        backreferences.push(current_pattern.clone());
                    }

                    if input_line_leftover.is_none() {
                        let mut input_line_clone = input_line.clone();

                        let is_conditional = current_pattern.starts_with(&['[']);
                        let char_after_group = if is_conditional {
                            pattern.get(current_pattern.len() + 2)
                        } else {
                            None
                        };

                        let current_pattern_match = match_pattern(
                            &mut input_line_clone,
                            &mut current_pattern,
                            must_match,
                            backreferences,
                            char_after_group,
                        );

                        if current_pattern_match {
                            matched_line = Some(
                                input_line[0..(input_line.len() - input_line_clone.len())].to_vec(),
                            );
                            input_line_leftover = Some(input_line_clone.clone());

                            input_line.clear();
                            input_line.extend(input_line_clone);
                        }
                    }

                    current_pattern.clear();

                    if pattern[i] == ')' {
                        break;
                    }
                } else {
                    current_pattern.push(pattern[i]);

                    if pattern[i] == ')' {
                        internal_parentheses_ends += 1;
                    }
                }


                i += 1;
            }
            
            
            if let Some(line) = input_line_leftover.as_mut() {
                let mut next_patterns = pattern[i + 1..].to_vec();
                backreferences.push(matched_line.unwrap());
                if next_patterns.len() > 0 {
                    let matched = match_pattern(line, &mut next_patterns, true, backreferences, char_after_group);

                    if matched {
                        input_line.clear();
                        input_line.extend(line.iter());
                    }

                    return matched;
                } else {
                    return true;
                }
            } else {
                return false;
            }
        }
        Some(x) => (vec![x.to_string()], 0usize, false),
        None => {
            return true;
        }
    };

    let (is_matching, input_line_matched_last_index, skip) = {
        let one_or_more = pattern
            .get(pattern_matched_last_index + 1)
            .map(|x| x == &'+')
            .unwrap_or(false);
        let zero_or_one = pattern
            .get(pattern_matched_last_index + 1)
            .map(|x| x == &'?')
            .unwrap_or(false);

        if one_or_more || zero_or_one {
            pattern_matched_last_index += 1;
        }

        let mut matches = 0;
        while let Some(next_char) = input_line.get(matches) {
            if match_next(*next_char, &conditions) {
                // If negated and match, it's not a global matching
                if is_negated {
                    return false;
                }

                matches += 1;

                if one_or_more {
                    continue;
                }
            } else if is_negated {
                if let Some(char_after_group) = char_after_group {
                    if *char_after_group == *next_char {
                        break;
                    }
                }

                matches += 1;

                if one_or_more {
                    continue;
                }
            }

            break;
        }

        // println!("MATCHES {:?}", matches);

        (
            matches > 0,
            matches.saturating_sub(1),
            zero_or_one && matches == 0,
        )
    };
    
    if !skip {
        input_line.drain(0..=input_line_matched_last_index);
    }

    if is_matching || skip {
        // Local match, remove matched pattern
        pattern.drain(0..=pattern_matched_last_index);

        // If no more pattern, it's a global match
        if pattern.len() == 0 {
            return true;
        }
    // This match was mandatory but it didn't happen
    } else if must_match {
        return false;
    }

    // If negated and no more input line, it's a global match
    if is_negated && input_line.len() == 0 {
        return true;
    }

    // Recursively call match_pattern with the rest of the input line
    match_pattern(
        input_line,
        pattern,
        must_match || is_matching,
        backreferences,
        char_after_group,
    )
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

    if match_pattern(&mut input_line, &mut pattern, false, &mut vec![], None) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

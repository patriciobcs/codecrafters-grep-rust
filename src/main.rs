use std::env;
use std::io;
use std::process;

/// Checks if a character matches any of the given conditions
/// 
/// This function is used to evaluate special character classes and literal characters
/// in the regex pattern.
fn match_next(next_char: char, conditions: &[String]) -> bool {
    conditions.iter().any(|condition| match condition.as_str() {
        "\\d" => next_char.is_digit(10),
        "\\s" => next_char.is_whitespace(),
        "\\w" => next_char.is_alphanumeric(),
        "." => true,
        x => next_char.to_string() == *x,
    })
}

/// The main regex matching function
/// 
/// This recursive function implements the core logic for matching a regex pattern
/// against an input string. It handles various regex features like anchors,
/// character classes, groups, and quantifiers.
fn match_pattern(
    input_line: &mut Vec<char>,
    pattern: &mut Vec<char>,
    must_match: bool,
    backreferences: &mut Vec<Vec<char>>,
    char_after_group: Option<&char>,
) -> bool {
    // If the input is empty, only match if the pattern ends with '$'
    if input_line.is_empty() {
        return pattern.first() == Some(&'$');
    }

    let mut input_line_leftover = None;

    // Handle different pattern start conditions
    let (conditions, mut pattern_matched_last_index, is_negated) = match pattern.first() {
        // Handle start-of-line anchor '^'
        Some(&'^') if !must_match => {
            return match_pattern(
                input_line,
                &mut pattern[1..].to_vec(),
                true,
                backreferences,
                None,
            )
        }
        // Handle escaped characters and backreferences
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
        // Handle character classes [...]
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
        // Handle groups (...)
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

                if internal_parentheses_starts == internal_parentheses_ends
                    && (pattern[i] == ')' || pattern[i] == '|')
                {
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
                    let matched = match_pattern(
                        line,
                        &mut next_patterns,
                        true,
                        backreferences,
                        char_after_group,
                    );

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
        // Handle literal characters
        Some(x) => (vec![x.to_string()], 0usize, false),
        // If pattern is empty, it's a match
        None => {
            return true;
        }
    };

    // Handle quantifiers and perform character matching
    let (is_matching, input_line_matched_last_index, skip) = {
        // Check for '+' and '?' quantifiers
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

        // Perform character matching
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

        // Return matching results
        (
            matches > 0,
            matches.saturating_sub(1),
            zero_or_one && matches == 0,
        )
    };

    // Remove matched characters from input, if a literal match happened
    if !skip {
        input_line.drain(0..=input_line_matched_last_index);
    }

    // Handle successful matches and pattern updates
    if is_matching || skip {
        // Remove matched pattern
        pattern.drain(0..=pattern_matched_last_index);

        // If no more pattern, it's a complete match
        if pattern.len() == 0 {
            return true;
        }
    // This match was mandatory but didn't happen
    } else if must_match {
        return false;
    }

    // Handle negated character classes at the end of input
    if is_negated && input_line.len() == 0 {
        return true;
    }

    // Recursively continue matching
    match_pattern(
        input_line,
        pattern,
        must_match || is_matching,
        backreferences,
        char_after_group,
    )
}

/// The main function that parses command-line arguments and runs the regex matcher
///
/// Usage: echo <input_text> | your_program -E <pattern>
fn main() {
    let args: Vec<String> = env::args().collect();

    // Ensure the first argument is '-E'
    if args.get(1).map_or(true, |arg| arg != "-E") {
        eprintln!("Expected first argument to be '-E'");
        process::exit(1);
    }

    // Get the regex pattern from command-line arguments
    let pattern: Vec<char> = args
        .get(2)
        .expect("Pattern argument is required")
        .chars()
        .collect();

    // Read input from stdin
    let input_line: Vec<char> = io::stdin()
        .lines()
        .next()
        .expect("Failed to read input")
        .expect("Failed to parse input")
        .chars()
        .collect();

    // Run the regex matcher and exit with appropriate status code
    process::exit(
        if match_pattern(
            &mut input_line.clone(),
            &mut pattern.clone(),
            false,
            &mut vec![],
            None,
        ) {
            0 // Exit with 0 if there's a match
        } else {
            1 // Exit with 1 if there's no match
        },
    );
}
use core::panic;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
struct NFA {
    transitions: HashMap<(u32, Option<char>), Vec<u32>>,
    accepting_state: u32, // the thompson construction always has one accepting_state
}

#[derive(Debug)]
struct DFA {
    transitions: HashMap<(u32, Option<char>), u32>,
    accepting_states: HashSet<u32>,
}

fn is_valid_regex(regex: &str) -> bool {
    if regex.is_empty() {
        return false;
    }

    let mut open_paren_count = 0;
    let mut last_was_quantifier = false;

    let mut chars = regex.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '(' => {
                open_paren_count += 1;
                last_was_quantifier = false;
            }

            ')' => {
                if open_paren_count == 0 {
                    return false;
                }
                open_paren_count -= 1;
                last_was_quantifier = false;
            }

            '*' | '+' => {
                // Ensure quantifiers are not the first character and are not repeated
                if last_was_quantifier || regex.starts_with('*') || regex.starts_with('+') {
                    return false;
                }
                last_was_quantifier = true;
            }

            '|' => {
                // Ensure alternation isn't the first or last character
                if regex.starts_with('|') || chars.peek().is_none() {
                    return false;
                }
                last_was_quantifier = false;
            }

            '\\' => {
                // Handle escaped characters: ensure there's a character after the escape
                if chars.peek().is_none() {
                    return false;
                }
                chars.next(); // Skip the escaped character
                last_was_quantifier = false;
            }

            _ => {
                last_was_quantifier = false;
            }
        }
    }

    open_paren_count == 0
}

fn normalize_regex(regex: &str) -> String {
    // TODO: Implement further parsing features here or in a separat function
    // e.g. a+ -> aa*
    if !is_valid_regex(regex) {
        panic!("{} is not a valid regular expression!", regex);
    }

    let mut normalized = String::new();
    let mut escape_sequence = false;
    let mut prev_char = '\0';

    for curr_char in regex.chars() {
        if escape_sequence {
            normalized.push(curr_char);
            escape_sequence = false;
            prev_char = curr_char;
            continue;
        }

        if curr_char == '\\' {
            escape_sequence = true;
            normalized.push(curr_char);
            continue;
        }

        if curr_char == '+' {
            normalized.push(prev_char);
            normalized.push('*');
            prev_char = curr_char;
            continue;
        }
        if curr_char == '?' {
            match prev_char {
                ')' => {
                    let mut balance = 0;

                    for j in (0..normalized.len()).rev() {
                        let ch = normalized.chars().nth(j).unwrap();
                        if ch == ')' {
                            balance += 1;
                        } else if ch == '(' {
                            balance -= 1;
                            if balance == 0 {
                                normalized.insert(j, '(');
                                break;
                            }
                        }
                    }
                }
                _ => {
                    normalized.insert(normalized.len() - 1, '(');
                }
            }
            normalized.push_str("|())");
            prev_char = curr_char;
            continue;
        }

        normalized.push(curr_char);
        prev_char = curr_char;
    }

    normalized
}

// TODO: Implement Glushkov Construction as well to benchmark them

// THOMPSON CONSTRUCTION ---
fn thompson_construction(normalized_regex: &str) -> NFA {
    let mut operators: Vec<char> = Vec::new();
    let mut nfa_concat_stack: Vec<NFA> = Vec::new();
    let mut tmp_concat_stack: Vec<NFA> = Vec::new();
    let mut concat_flag: bool = false;
    let mut escape_sequence = false;

    for letter in normalized_regex.chars() {
        if escape_sequence {
            // TODO: Handle escape sequence e.g. \w -> [a-zA-Z]
            // for now however it will just be a normal letter e.g. escaping \|
            nfa_concat_stack.push(create_basic_nfa(&letter));
            escape_sequence = false;
            continue;
        }

        match letter {
            '\\' => escape_sequence = true,
            '*' => {
                let last_nfa = nfa_concat_stack
                    .pop()
                    .expect("Expected NFA for Kleene Star");
                nfa_concat_stack.push(apply_kleene_star(&last_nfa));
                concat_flag = true;
            }
            '|' => {
                operators.push('|');
                concat_flag = false;
            }
            '(' => {
                concat_flag = false;
                while let Some(&top_operator) = operators.last() {
                    if top_operator == '(' {
                        break;
                    }

                    let op: Option<char> = operators.pop();

                    if op == Some('.') {
                        let nfa_left = nfa_concat_stack
                            .pop()
                            .expect("Expected NFA for concatenation");
                        let nfa_right = nfa_concat_stack
                            .pop()
                            .expect("Expected NFA for concatenation");
                        nfa_concat_stack.push(concatenate(&nfa_left, &nfa_right));
                    } else if op == Some('|') {
                        let nfa_right = nfa_concat_stack.pop().expect("Expected NFA for union");
                        if operators.last() == Some(&'.') {
                            tmp_concat_stack
                                .push(nfa_concat_stack.pop().expect("Expected an operand"));

                            while operators.last() == Some(&'.') {
                                tmp_concat_stack
                                    .push(nfa_concat_stack.pop().expect("Expected an operand"));
                                operators.pop(); // Remove the '.' operator
                            }

                            let mut nfa_left = concatenate(
                                &tmp_concat_stack
                                    .pop()
                                    .expect("Expected NFA from concat stack"),
                                &tmp_concat_stack
                                    .pop()
                                    .expect("Expected NFA from concat stack"),
                            );

                            while !tmp_concat_stack.is_empty() {
                                nfa_left = concatenate(
                                    &nfa_left,
                                    &tmp_concat_stack
                                        .pop()
                                        .expect("Expected NFA from concat stack"),
                                );
                            }

                            nfa_concat_stack.push(union(&nfa_left, &nfa_right));
                        } else {
                            if let Some(nfa_left) = nfa_concat_stack.pop() {
                                nfa_concat_stack.push(union(&nfa_left, &nfa_right));
                            } else {
                                panic!("Expected operand to form the union!");
                            }
                        }
                    } else {
                        panic!("Expected at least one operand to process NFA");
                    }
                }
            }
            ')' => {
                continue;
            }
            _ => {
                // Create a basic NFA for single character
                nfa_concat_stack.push(create_basic_nfa(&letter));
                if concat_flag {
                    operators.push('.');
                } else {
                    concat_flag = true;
                }
            }
        }
    }

    while !operators.is_empty() {
        let mut nfa_left;
        let op: Option<char> = operators.pop();
        if op == Some('.') {
            let nfa_right = nfa_concat_stack
                .pop()
                .expect("Expected NFA for concatenation");
            let nfa_left = nfa_concat_stack
                .pop()
                .expect("Expected NFA for concatenation");
            nfa_concat_stack.push(concatenate(&nfa_left, &nfa_right));
        } else if op == Some('|') {
            let nfa_right = nfa_concat_stack
                .pop()
                .expect("Expected NFA for concatenation");
            if operators.last() == Some(&'.') {
                tmp_concat_stack.push(
                    nfa_concat_stack
                        .pop()
                        .expect("Expected a NFA for temporary concat stack"),
                );
                while operators.last() == Some(&'.') {
                    tmp_concat_stack.push(
                        nfa_concat_stack
                            .pop()
                            .expect("Expected a NFA for temporary concat stack"),
                    );
                    operators.pop();
                }
                nfa_left = concatenate(
                    &tmp_concat_stack
                        .pop()
                        .expect("Expected a NFA for the concatenation"),
                    &tmp_concat_stack
                        .pop()
                        .expect("Expected a NFA for the concatenation"),
                );
                while !tmp_concat_stack.is_empty() {
                    nfa_left = concatenate(
                        &nfa_left,
                        &tmp_concat_stack
                            .pop()
                            .expect("Expected a NFA for the concatenation"),
                    );
                }
            } else {
                nfa_left = nfa_concat_stack
                    .pop()
                    .expect("Expected a NFA for the union");
            }
            nfa_concat_stack.push(union(&nfa_left, &nfa_right));
        }
    }

    if nfa_concat_stack.len() != 1 {
        panic!("Invalid Regex, unexpected final NFA stack size");
    }

    nfa_concat_stack.pop().unwrap()
}

fn apply_kleene_star(last_nfa: &NFA) -> NFA {
    let mut transitions = HashMap::new();

    let new_accepting = last_nfa.accepting_state + 2;

    // Epsilon transition from new start to original start
    transitions.insert((0, None), vec![1]);

    // Copy existing transitions, shifting state numbers to make room for new start
    for ((state, input), targets) in &last_nfa.transitions {
        // Shift each transition to new indices
        transitions.insert((state + 1, *input), targets.iter().map(|s| s + 1).collect());
    }

    // Epsilon transitions returning to original start for loops, and new accepting state
    transitions
        .entry((&last_nfa.accepting_state + 1, None))
        .or_insert_with(Vec::new)
        .push(1);

    transitions
        .entry((&last_nfa.accepting_state + 1, None))
        .or_insert_with(Vec::new)
        .push(new_accepting);

    // Final acceptance state is accepting with epsilon transition from start for empty string
    transitions
        .entry((0, None))
        .or_insert_with(Vec::new)
        .push(new_accepting);

    NFA {
        transitions,
        accepting_state: new_accepting,
    }
}

fn union(left: &NFA, right: &NFA) -> NFA {
    let mut transitions = HashMap::new();

    let num_states_left_nfa = left.accepting_state;
    let num_states_right_nfa = right.accepting_state;

    // Shift the NFA states
    for ((state, input), targets) in &left.transitions {
        transitions.insert((state + 1, *input), targets.iter().map(|s| s + 1).collect());
    }

    for ((state, input), targets) in &right.transitions {
        transitions.insert(
            (state + num_states_left_nfa + 2, *input),
            targets
                .iter()
                .map(|s| s + num_states_left_nfa + 2)
                .collect(),
        );
    }

    // Add new start and end state
    let new_accepting_state = num_states_left_nfa + num_states_right_nfa + 3;

    transitions.insert((0, None), vec![1, num_states_left_nfa + 2]);
    transitions
        .entry((&left.accepting_state + 1, None))
        .or_insert_with(Vec::new)
        .push(new_accepting_state);
    transitions
        .entry((&right.accepting_state + num_states_left_nfa + 2, None))
        .or_insert_with(Vec::new)
        .push(new_accepting_state);

    NFA {
        transitions,
        accepting_state: new_accepting_state,
    }
}

fn concatenate(left: &NFA, right: &NFA) -> NFA {
    let mut transitions = HashMap::from(left.transitions.clone());

    // HACK: The accepting states are (based on the implementation) the last ones of the NFA
    // thus it is possible to get the num of states in the first NFA like this
    let num_states_left_nfa = left.accepting_state;

    for ((state, input), targets) in &right.transitions {
        transitions.insert(
            (state + num_states_left_nfa, *input),
            targets.iter().map(|s| s + num_states_left_nfa).collect(),
        );
    }

    NFA {
        transitions,
        accepting_state: right.accepting_state + num_states_left_nfa,
    }
}

fn create_basic_nfa(letter: &char) -> NFA {
    NFA {
        transitions: HashMap::from([((0, Some(*letter)), vec![1])]),
        accepting_state: 1,
    }
}
// END THOMPSON CONSTRUCTION ---

fn nfa_to_dfa(regex_nfa: &NFA) -> DFA {
    unimplemented!()
}

impl DFA {
    // TODO: maybe do from NFA instead of regex
    fn new(regex: &str) -> Self {
        // TODO: Maybe optimize regex
        // TODO: Normalize Regex by removing escape sequenzes etc.
        // -> Regex only consisting of () | * and a . for seperating the groups afterwards
        let normalized_regex = normalize_regex(&regex);
        // TODO: Implement Thompson construction
        let regex_nfa: NFA = thompson_construction(&normalized_regex);
        // TODO: Maybe optimize nfa
        // TODO: Converting NFA to DFA
        nfa_to_dfa(&regex_nfa)
        // TODO: Optimize dfa
    }

    pub fn process(&self, input: &str) -> bool {
        let mut current_state = 0;
        for c in input.chars() {
            if let Some(&next_state) = self.transitions.get(&(current_state, Some(c))) {
                current_state = next_state;
            } else {
                return false;
            }
        }
        self.accepting_states.contains(&current_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_regex_basic_test() {
        let regex = "(a|b)*";
        assert!(is_valid_regex(regex), "Expected valid regex.");
    }

    #[test]
    fn invalid_empty_regex_test() {
        let regex = "";
        assert!(!is_valid_regex(regex), "Expected invalid regex (empty).");
    }

    #[test]
    fn invalid_unbalanced_parentheses_test() {
        let regex1 = "(a|b";
        let regex2 = "a|b)";
        assert!(
            !is_valid_regex(regex1),
            "Expected invalid regex (unbalanced parentheses)."
        );
        assert!(
            !is_valid_regex(regex2),
            "Expected invalid regex (unbalanced parentheses)."
        );
    }

    #[test]
    fn invalid_operator_placement_test() {
        let regex1 = "*a";
        let regex2 = "|a|b";
        assert!(
            !is_valid_regex(regex1),
            "Expected invalid regex (invalid quantifier placement)."
        );
        assert!(
            !is_valid_regex(regex2),
            "Expected invalid regex (invalid alternation placement)."
        );
    }

    #[test]
    fn valid_nested_parentheses_test() {
        let regex = "((a|b)*c)";
        assert!(
            is_valid_regex(regex),
            "Expected valid regex with nested parentheses."
        );
    }

    #[test]
    fn valid_escape_sequence_test() {
        let regex = "a\\*b";
        assert!(
            is_valid_regex(regex),
            "Expected valid regex with escape sequence."
        );
    }

    #[test]
    fn invalid_escape_sequence_test() {
        let regex = "a\\";
        assert!(
            !is_valid_regex(regex),
            "Expected invalid regex with unpaired escape."
        );
    }

    #[test]
    fn normalize_regex_test() {
        let cases = [
            (r"a+", r"aa*"),
            (r"a\+", r"a\+"),
            (r"a?", r"(a|())"),
            (r"a\?", r"a\?"),
            (r"(ab)?", r"((ab)|())"),
        ];

        for (input, expected) in cases {
            let result = normalize_regex(input);
            assert_eq!(
                result, expected,
                "Normalization failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn create_dfa_test() {
        unimplemented!();
    }

    #[test]
    fn prozess_regex_test() {
        unimplemented!();
    }

    #[test]
    fn thompson_construction_test() {
        let regex_nfa = thompson_construction("(a|b)*");
        let expected_nfa = NFA {
            transitions: HashMap::from([
                ((0, None), vec![1, 3]),
                ((1, Some('a')), vec![2]),
                ((2, None), vec![3]),
                ((3, Some('b')), vec![4]),
                ((4, None), vec![3]),
            ]),
            accepting_state: 4,
        };
        assert_eq!(regex_nfa, expected_nfa);
    }

    #[test]
    fn create_basic_nfa_test() {
        let nfa_a = create_basic_nfa(&'a');
        let expected_transitions = HashMap::from([((0, Some('a')), vec![1])]);
        let expected_accepting_state: u32 = 1;

        assert_eq!(nfa_a.transitions, expected_transitions);
        assert_eq!(nfa_a.accepting_state, expected_accepting_state);
    }

    #[test]
    fn concatenate_test() {
        let nfa_a = create_basic_nfa(&'a');
        let nfa_b = create_basic_nfa(&'b');
        let concatenated_nfa = concatenate(&nfa_a, &nfa_b);

        let expected_transitions =
            HashMap::from([((0, Some('a')), vec![1]), ((1, Some('b')), vec![2])]);
        let expected_accepting_state: u32 = 2;

        assert_eq!(concatenated_nfa.transitions, expected_transitions);
        assert_eq!(concatenated_nfa.accepting_state, expected_accepting_state);
    }

    #[test]
    fn apply_kleene_star_test() {
        let basic_nfa = create_basic_nfa(&'a');
        let starred_nfa = apply_kleene_star(&basic_nfa);

        let expected_transitions = HashMap::from([
            ((0, None), vec![1, 3]),   // Epsilon to start and new accepting
            ((1, Some('a')), vec![2]), // Original transition
            ((2, None), vec![1, 3]),   // Loop back and transition to new accepting
        ]);

        let expected_accepting_state: u32 = 3;

        assert_eq!(starred_nfa.transitions, expected_transitions);
        assert_eq!(starred_nfa.accepting_state, expected_accepting_state);
    }

    #[test]
    fn union_test() {
        let nfa_a = create_basic_nfa(&'a');
        let nfa_b = create_basic_nfa(&'b');
        let union_nfa = union(&nfa_a, &nfa_b);

        let expected_transitions = HashMap::from([
            ((0, None), vec![1, 3]),   // Combined initial state transitions
            ((1, Some('a')), vec![2]), // Offset transitions for NFA a
            ((3, Some('b')), vec![4]), // Offset transitions for NFA b
            ((2, None), vec![5]),      // Accepting state transition for a
            ((4, None), vec![5]),      // Accepting state transition for b
        ]);

        let expected_accepting_state: u32 = 5;

        assert_eq!(union_nfa.transitions, expected_transitions);
        assert_eq!(union_nfa.accepting_state, expected_accepting_state);
    }

    #[test]
    fn nfa_to_dfa_test() {
        unimplemented!();
    }
}

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct NFA {
    alphabet: HashSet<char>,
    transitions: HashMap<(u32, char), Vec<u32>>,
    accepting_states: HashSet<u32>,
}

#[derive(Debug)]
struct DFA {
    alphabet: HashSet<char>,
    transitions: HashMap<(u32, char), u32>,
    accepting_states: HashSet<u32>,
}

fn normalize_regex(regex: &str) -> String {
    // TODO: Implement further parsing features here or in a separat function
    // e.g. a+ -> aa*
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

        normalized.push(curr_char);
        prev_char = curr_char;
    }

    normalized
}

fn thompson_construction(normalized_regex: &str) -> NFA {
    let mut created_nfa = NFA {
        alphabet: HashSet::new(),
        transitions: HashMap::new(),
        accepting_states: HashSet::new(),
    };

    let mut stack: Vec<NFA> = Vec::new();
    let mut escape_sequence = false;
    let mut prev_char = '\0';

    for letter in normalized_regex.chars() {
        if escape_sequence {
            // TODO: Handle escape sequence
            escape_sequence = false;
            continue;
        }

        match letter {
            '\\' => escape_sequence = true,
            '*' => {
                let last_nfa = stack.pop().expect("Expected NFA for Kleene Star");
                stack.push(apply_kleene_star(&last_nfa));
            }
            '|' => {
                // Apply union to the last two NFAs
                let right = stack.pop().expect("Expected NFA for union");
                let left = stack.pop().expect("Expected NFA for union");
                stack.push(union(&left, &right));
            }
            '(' | ')' => {
                // Handle parentheses using the stack, typically you'd handle these with a more complex mechanism.
                unimplemented!();
            }
            _ => {
                // Create a basic NFA for single character
                stack.push(create_basic_nfa(&letter));

                if prev_char != '\0'
                    && ((prev_char.is_alphanumeric() && letter.is_alphanumeric()) ||    // Consecutive literals
                    (prev_char.is_alphanumeric() && letter == '(') ||                   // Literal + opening parenthesis
                    (prev_char == ')' && letter.is_alphanumeric()) ||                   // Closing parenthesis + literal
                    (prev_char == '*' && letter.is_alphanumeric()))
                {
                    let right = stack.pop().expect("Expected NFA for concatenation");
                    let left = stack.pop().expect("Expected NFA for concatenation");
                    stack.push(concatenate(&left, &right));
                }
            }
        }

        created_nfa.alphabet.insert(letter);
    }

    if stack.len() != 1 {
        panic!("Invalid Regex, unexpected final NFA stack size");
    }

    stack.pop().unwrap()
}

fn apply_kleene_star(last_nfa: &NFA) -> NFA {
    unimplemented!()
}

fn union(left: &NFA, right: &NFA) -> NFA {
    unimplemented!()
}

fn concatenate(left: &NFA, right: &NFA) -> NFA {
    unimplemented!()
}

fn create_basic_nfa(letter: &char) -> NFA {
    unimplemented!()
}

fn nfa_to_dfa(regex_nfa: &NFA) -> DFA {
    unimplemented!()
}

impl DFA {
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
            if !self.alphabet.contains(&c) {
                return false;
            }
            if let Some(&next_state) = self.transitions.get(&(current_state, c)) {
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
    fn test_normalize_regex() {
        let cases = [(r"a+", r"aa*"), (r"a\+", r"a\+")];

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
    fn prozess_function_test() {
        unimplemented!();
    }

    #[test]
    fn thompson_construction_test() {
        unimplemented!();
    }

    #[test]
    fn nfa_to_dfa_test() {
        unimplemented!();
    }
}

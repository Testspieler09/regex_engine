use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct NFA {
    states: HashSet<u32>,
    alphabet: HashSet<char>,
    transitions: HashMap<(u32, char), Vec<u32>>,
    start_state: u32,
    accepting_states: HashSet<u32>,
}

#[derive(Debug)]
struct DFA {
    states: HashSet<u32>,
    alphabet: HashSet<char>,
    transitions: HashMap<(u32, char), u32>,
    start_state: u32,
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
            normalized.push('.');
            normalized.push(curr_char);
            continue;
        }

        // Insert concatenation operator if needed
        if prev_char != '\0'
            && ((prev_char.is_alphanumeric() && curr_char.is_alphanumeric()) || // Consecutive literals
                (prev_char.is_alphanumeric() && curr_char == '(') ||            // Literal + opening parenthesis
                (prev_char == ')' && curr_char.is_alphanumeric()) ||            // Closing parenthesis + literal
                (prev_char == '*' && curr_char.is_alphanumeric())) {            // Closing parenthesis + unary operator
            normalized.push('.');
        }

        normalized.push(curr_char);
        prev_char = curr_char;
    }

    normalized
}

fn thompson_construction(normalized_regex: &str) -> NFA {
    let mut created_nfa = NFA {
        states: HashSet::new(),
        alphabet: HashSet::new(),
        transitions: HashMap::new(),
        start_state: 0,
        accepting_states: HashSet::new(),
    };

    let mut stack: Vec<NFA> = Vec::new();
    let mut escape_sequence = false;

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
            },
            '.' => {
                // Concatenate the last two NFAs
                let right = stack.pop().expect("Expected NFA for concatenation");
                let left = stack.pop().expect("Expected NFA for concatenation");
                stack.push(concatenate(&left, &right));
            },
            '|' => {
                // Apply union to the last two NFAs
                let right = stack.pop().expect("Expected NFA for union");
                let left = stack.pop().expect("Expected NFA for union");
                stack.push(union(&left, &right));
            },
            '(' | ')' => {
                // Handle parentheses using the stack, typically you'd handle these with a more complex mechanism.
                unimplemented!();
            },
            _ => {
                // Create a basic NFA for single character
                stack.push(create_basic_nfa(&letter));
            },
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
    // TODO: implement this
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
        let mut current_state = self.start_state;
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
        let cases = [
            (r"a(b|c)*d", r"a.(b|c)*.d"),
            (r"ab", r"a.b"),
            (r"abc", r"a.b.c"),
            (r"(ab)c", r"(a.b).c"),
            (r"a|(b*c)", r"a|(b*.c)"),
            (r"", r""),
            (r"a\*b", r"a.\*.b"),
        ];

        for (input, expected) in cases {
            let result = normalize_regex(input);
            assert_eq!(result, expected, "Normalization failed for input '{}'", input);
        }
    }

    #[test]
    fn create_dfa_test() {
        todo!();
    }

    #[test]
    fn prozess_function_test() {
        todo!();
    }

    #[test]
    fn thompson_construction_test() {
        todo!();
    }

    #[test]
    fn nfa_to_dfa_test() {
        todo!();
    }
}

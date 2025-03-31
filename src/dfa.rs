use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct NFA {
    transitions: HashMap<(u32, Option<char>), Vec<u32>>,
    accepting_states: HashSet<u32>,
}

#[derive(Debug)]
struct DFA {
    transitions: HashMap<(u32, Option<char>), u32>,
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

// TODO: implement Glushkov Construction as well to benchmark them

// THOMPSON CONSTRUCTION ---
fn thompson_construction(normalized_regex: &str) -> NFA {
    let mut stack: Vec<NFA> = Vec::new();
    let mut escape_sequence = false;
    let mut prev_char = '\0';

    for letter in normalized_regex.chars() {
        if escape_sequence {
            // TODO: Handle escape sequence e.g. \w -> [a-zA-Z]
            // for now however it will just be a normal letter e.g. escaping \|
            stack.push(create_basic_nfa(&letter));
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
                // FIX: Apply union to the last NFA and the next one instead of the last two ones
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
        // TODO: merge the stack into one nfa?!
    }

    if stack.len() != 1 {
        panic!("Invalid Regex, unexpected final NFA stack size");
    }

    stack.pop().unwrap()
}

fn apply_kleene_star(last_nfa: &NFA) -> NFA {
    let mut transitions = HashMap::new();
    let mut accepting_states = HashSet::new();

    let new_accepting = last_nfa.accepting_states.iter().max().unwrap() + 2;

    // Epsilon transition from new start to original start
    transitions.insert((0, None), vec![1]);

    // Copy existing transitions, shifting state numbers to make room for new start
    for ((state, input), targets) in &last_nfa.transitions {
        // Shift each transition to new indices
        transitions.insert((state + 1, *input), targets.iter().map(|s| s + 1).collect());
    }

    // Epsilon transitions returning to original start for loops, and new accepting state
    for &accepting_state in &last_nfa.accepting_states {
        transitions
            .entry((accepting_state + 1, None))
            .or_insert_with(Vec::new)
            .push(1);

        transitions
            .entry((accepting_state + 1, None))
            .or_insert_with(Vec::new)
            .push(new_accepting);
    }

    // Final acceptance state is accepting with epsilon transition from start for empty string
    accepting_states.insert(new_accepting);
    transitions
        .entry((0, None))
        .or_insert_with(Vec::new)
        .push(new_accepting);

    NFA {
        transitions,
        accepting_states,
    }
}

fn union(left: &NFA, right: &NFA) -> NFA {
    let mut transitions = HashMap::new();

    let num_states_left_nfa = left.accepting_states.iter().max().unwrap();
    let num_states_right_nfa = right.accepting_states.iter().max().unwrap();

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
    for &accepting_state in &left.accepting_states {
        transitions
            .entry((accepting_state + 1, None))
            .or_insert_with(Vec::new)
            .push(new_accepting_state);
    }
    for &accepting_state in &right.accepting_states {
        transitions
            .entry((accepting_state + num_states_left_nfa + 2, None))
            .or_insert_with(Vec::new)
            .push(new_accepting_state);
    }

    NFA {
        transitions,
        accepting_states: HashSet::from([new_accepting_state]),
    }
}

fn concatenate(left: &NFA, right: &NFA) -> NFA {
    let mut transitions = HashMap::from(left.transitions.clone());

    // HACK: The accepting states are (based on the implementation) the last ones of the NFA
    // thus it is possible to get the num of states in the first NFA like this
    let num_states_left_nfa = left.accepting_states.iter().max().unwrap();

    for ((state, input), targets) in &right.transitions {
        transitions.insert(
            (state + num_states_left_nfa, *input),
            targets.iter().map(|s| s + num_states_left_nfa).collect(),
        );
    }

    NFA {
        transitions,
        accepting_states: HashSet::from(
            right
                .accepting_states
                .iter()
                .map(|s| s + num_states_left_nfa)
                .collect::<HashSet<_>>(),
        ),
    }
}

fn create_basic_nfa(letter: &char) -> NFA {
    NFA {
        transitions: HashMap::from([((0, Some(*letter)), vec![1])]),
        accepting_states: HashSet::from([1]),
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
    fn create_basic_nfa_test() {
        let nfa_a = create_basic_nfa(&'a');
        let expected_transitions = HashMap::from([((0, Some('a')), vec![1])]);
        let expected_accepting_states: HashSet<u32> = HashSet::from([1]);

        assert_eq!(nfa_a.transitions, expected_transitions);
        assert_eq!(nfa_a.accepting_states, expected_accepting_states);
    }

    #[test]
    fn concatenate_test() {
        let nfa_a = create_basic_nfa(&'a');
        let nfa_b = create_basic_nfa(&'b');
        let concatenated_nfa = concatenate(&nfa_a, &nfa_b);

        let expected_transitions =
            HashMap::from([((0, Some('a')), vec![1]), ((1, Some('b')), vec![2])]);
        let expected_accepting_states: HashSet<u32> = HashSet::from([2]);

        assert_eq!(concatenated_nfa.transitions, expected_transitions);
        assert_eq!(concatenated_nfa.accepting_states, expected_accepting_states);
    }

    #[test]
    fn test_apply_kleene_star() {
        let basic_nfa = create_basic_nfa(&'a');
        let starred_nfa = apply_kleene_star(&basic_nfa);

        let expected_transitions = HashMap::from([
            ((0, None), vec![1, 3]),   // Epsilon to start and new accepting
            ((1, Some('a')), vec![2]), // Original transition
            ((2, None), vec![1, 3]),   // Loop back and transition to new accepting
        ]);

        let expected_accepting_states: HashSet<u32> = HashSet::from([3]);

        assert_eq!(starred_nfa.transitions, expected_transitions);
        assert_eq!(starred_nfa.accepting_states, expected_accepting_states);
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

        let expected_accepting_states: HashSet<u32> = HashSet::from([5]);

        assert_eq!(union_nfa.transitions, expected_transitions);
        assert_eq!(union_nfa.accepting_states, expected_accepting_states);
    }

    #[test]
    fn nfa_to_dfa_test() {
        unimplemented!();
    }
}

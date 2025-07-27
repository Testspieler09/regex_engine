use crate::{Dfa, is_valid_regex, normalise_regex};
use std::collections::{HashMap, HashSet};

struct Nfa {
    transitions: HashMap<(u32, Option<char>), Vec<u32>>,
    accepting_state: u32, // the thompson construction always has one accepting_state
}

pub struct ThompsonDfa {
    transitions: HashMap<(u32, char), u32>,
    accepting_states: HashSet<u32>,
}

impl Dfa for ThompsonDfa {
    fn new(regex: &str) -> Self {
        if !is_valid_regex(regex) {
            panic!("{regex} is not a valid regular expression!");
        }

        let normalised_regex = normalise_regex(regex);
        let regex_nfa: Nfa = thompson_construction(&normalised_regex);
        let mut regex_dfa = nfa_to_dfa(&regex_nfa);
        <Self as Dfa>::optimise_dfa(&mut regex_dfa);
        regex_dfa
    }

    fn get_transitions(&self) -> &HashMap<(u32, char), u32> {
        &self.transitions
    }

    fn get_accepting_states(&self) -> &HashSet<u32> {
        &self.accepting_states
    }

    fn get_transitions_mut(&mut self) -> &mut HashMap<(u32, char), u32> {
        &mut self.transitions
    }

    fn get_accepting_states_mut(&mut self) -> &mut HashSet<u32> {
        &mut self.accepting_states
    }
}

// THOMPSON CONSTRUCTION ---
fn thompson_construction(normalised_regex: &str) -> Nfa {
    fn apply_operator(nfa_stack: &mut Vec<Nfa>, operator: char) {
        match operator {
            '|' => {
                let nfa_right = nfa_stack.pop().expect("Expected NFA for union");
                let nfa_left = nfa_stack.pop().expect("Expected NFA for union");
                nfa_stack.push(union(&nfa_left, &nfa_right));
            }
            '.' => {
                let nfa_right = nfa_stack.pop().expect("Expected NFA for concatenation");
                let nfa_left = nfa_stack.pop().expect("Expected NFA for concatenation");
                nfa_stack.push(concatenate(&nfa_left, &nfa_right));
            }
            _ => unreachable!("Unknown operator {}", operator),
        }
    }

    let mut operators: Vec<char> = Vec::new();
    let mut nfa_stack: Vec<Nfa> = Vec::new();
    let mut concat_flag = false;
    let mut escape_sequence = false;

    for symbol in normalised_regex.chars() {
        if escape_sequence {
            if concat_flag {
                operators.push('.');
            }
            nfa_stack.push(create_basic_nfa(&symbol));
            concat_flag = true;
            escape_sequence = false;
            continue;
        }

        match symbol {
            '(' => {
                if concat_flag {
                    operators.push('.');
                }
                operators.push('(');
                concat_flag = false;
            }
            ')' => {
                // If concat_flag is false, we have an empty right operand for union
                if !concat_flag {
                    nfa_stack.push(create_basic_epsilon_nfa());
                }

                // Process all operators until we hit the matching '('
                while let Some(op) = operators.pop() {
                    if op == '(' {
                        break;
                    }
                    apply_operator(&mut nfa_stack, op);
                }

                // If stack is empty after processing, we had completely empty parentheses
                if nfa_stack.is_empty() {
                    nfa_stack.push(create_basic_epsilon_nfa());
                }

                concat_flag = true;
            }
            '*' => {
                let last_nfa = nfa_stack.pop().expect("Expected NFA for Kleene Star");
                nfa_stack.push(apply_kleene_star(&last_nfa));
                concat_flag = true;
            }
            '|' => {
                // Process all concatenation operators (higher precedence than union)
                while let Some(&op) = operators.last() {
                    if op == '(' || op == '|' {
                        break;
                    }
                    operators.pop();
                    apply_operator(&mut nfa_stack, op);
                }

                // If we have no operand for the left side of union, create epsilon
                if !concat_flag {
                    nfa_stack.push(create_basic_epsilon_nfa());
                }

                operators.push('|');
                concat_flag = false;
            }
            '\\' => {
                escape_sequence = true;
            }
            _ => {
                if concat_flag {
                    operators.push('.');
                }
                nfa_stack.push(create_basic_nfa(&symbol));
                concat_flag = true;
            }
        }
    }

    // Handle case where regex ends with '|' (empty right operand)
    if let Some(&'|') = operators.last() {
        if nfa_stack.len() < 2 {
            nfa_stack.push(create_basic_epsilon_nfa());
        }
    }

    // Process remaining operators
    while let Some(op) = operators.pop() {
        if op == '(' {
            panic!("Unmatched opening parenthesis");
        }
        apply_operator(&mut nfa_stack, op);
    }

    if nfa_stack.len() != 1 {
        panic!(
            "Invalid Regex, unexpected final NFA stack size: {}",
            nfa_stack.len()
        );
    }

    nfa_stack.pop().unwrap()
}

fn apply_kleene_star(last_nfa: &Nfa) -> Nfa {
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

    Nfa {
        transitions,
        accepting_state: new_accepting,
    }
}

fn union(left: &Nfa, right: &Nfa) -> Nfa {
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
        .entry((left.accepting_state + 1, None))
        .or_insert_with(Vec::new)
        .push(new_accepting_state);
    transitions
        .entry((right.accepting_state + num_states_left_nfa + 2, None))
        .or_insert_with(Vec::new)
        .push(new_accepting_state);

    Nfa {
        transitions,
        accepting_state: new_accepting_state,
    }
}

fn concatenate(left: &Nfa, right: &Nfa) -> Nfa {
    let mut transitions: HashMap<(u32, Option<char>), Vec<u32>> = left.transitions.clone();

    // HACK: The accepting states are (based on the implementation) the last ones of the NFA
    // thus it is possible to get the num of states in the first NFA like this
    let num_states_left_nfa = left.accepting_state;

    for ((state, input), targets) in &right.transitions {
        transitions.insert(
            (state + num_states_left_nfa, *input),
            targets.iter().map(|s| s + num_states_left_nfa).collect(),
        );
    }

    Nfa {
        transitions,
        accepting_state: right.accepting_state + num_states_left_nfa,
    }
}

fn create_basic_nfa(letter: &char) -> Nfa {
    Nfa {
        transitions: HashMap::from([((0, Some(*letter)), vec![1])]),
        accepting_state: 1,
    }
}

fn create_basic_epsilon_nfa() -> Nfa {
    Nfa {
        transitions: HashMap::from([((0, None), vec![1])]),
        accepting_state: 1,
    }
}
// END THOMPSON CONSTRUCTION ---

// NFA to DFA functions ---
fn epsilon_closure(nfa: &Nfa, states: &mut HashSet<u32>) {
    let mut stack = states.clone();

    while let Some(&state_id) = stack.iter().next() {
        stack.remove(&state_id);
        if let Some(epsilon_states) = nfa.transitions.get(&(state_id, None)) {
            for &next_state in epsilon_states {
                if states.insert(next_state) {
                    stack.insert(next_state);
                }
            }
        }
    }
}

fn move_nfa(nfa: &Nfa, states: &HashSet<u32>, symbol: char) -> HashSet<u32> {
    let mut move_states = HashSet::new();

    for &state in states {
        if let Some(next_states) = nfa.transitions.get(&(state, Some(symbol))) {
            move_states.extend(next_states);
        }
    }

    move_states
}

fn hash_set_to_sorted_vec(set: &HashSet<u32>) -> Vec<u32> {
    let mut vec: Vec<u32> = set.iter().cloned().collect();
    vec.sort_unstable();
    vec
}

fn nfa_to_dfa(nfa: &Nfa) -> ThompsonDfa {
    // Start from the initial state of the NFA, assuming it's state 0
    let mut start_closure = HashSet::from([0]);
    epsilon_closure(nfa, &mut start_closure);
    let mut state_map = HashMap::new();
    let mut dfa_accepting_states = HashSet::new();
    let mut transitions = HashMap::new();

    // Map the initial DFA state from the initial NFA state closure
    state_map.insert(hash_set_to_sorted_vec(&start_closure), 0);

    let mut unmarked_states = vec![start_closure];

    while let Some(current_closure) = unmarked_states.pop() {
        let current_dfa_state_id = state_map[&hash_set_to_sorted_vec(&current_closure)];

        if current_closure.contains(&nfa.accepting_state) {
            dfa_accepting_states.insert(current_dfa_state_id);
        }

        // Collect symbols from transitions
        let symbols: HashSet<_> = nfa
            .transitions
            .keys()
            .filter_map(|(_, symbol)| *symbol)
            .collect();

        for symbol in symbols {
            let mut move_closure = move_nfa(nfa, &current_closure, symbol);
            epsilon_closure(nfa, &mut move_closure);

            if move_closure.is_empty() {
                continue;
            }

            let sorted_vec = hash_set_to_sorted_vec(&move_closure);
            let next_dfa_state_id = state_map.len() as u32;

            // Insert new DFA state if isn't already mapped
            if !state_map.contains_key(&sorted_vec) {
                state_map.insert(sorted_vec.clone(), next_dfa_state_id);
                unmarked_states.push(move_closure);
            }

            transitions.insert((current_dfa_state_id, symbol), state_map[&sorted_vec]);
        }
    }

    ThompsonDfa {
        transitions,
        accepting_states: dfa_accepting_states,
    }
}
// END NFA to DFA functions ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_dfa_test() {
        let generated_dfa = ThompsonDfa::new("(a|b)*");
        let expected_transitions = HashMap::from([((0, 'a'), 0), ((0, 'b'), 0)]);
        let expected_accepting_states = HashSet::from([0]);

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);

        let generated_dfa_2 = ThompsonDfa::new("a|()");
        let expected_transitions_2 = HashMap::from([((0, 'a'), 1)]);
        let expected_accepting_states_2 = HashSet::from([0, 1]);

        assert_eq!(expected_transitions_2, generated_dfa_2.transitions);
        assert_eq!(
            expected_accepting_states_2,
            generated_dfa_2.accepting_states
        );

        let generated_dfa = ThompsonDfa::new("a*b");
        let expected_transitions = HashMap::from([((0, 'a'), 0), ((0, 'b'), 1)]);
        let expected_accepting_states = HashSet::from([1]);

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }

    #[test]
    fn prozess_regex_test() {
        let generated_dfa = ThompsonDfa::new("(a|b)*");
        let test_strings = vec!["abbbababaaaa", ""];
        for string in test_strings {
            assert!(generated_dfa.process(string));
        }
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
    fn thompson_construction_test() {
        let regex_nfa = thompson_construction("(a|b)*");

        let expected_transitions = HashMap::from([
            ((0, None), vec![1, 7]),
            ((1, None), vec![2, 4]),
            ((2, Some('a')), vec![3]),
            ((3, None), vec![6]),
            ((4, Some('b')), vec![5]),
            ((5, None), vec![6]),
            ((6, None), vec![1, 7]),
        ]);
        let expected_accepting_state = 7;

        assert_eq!(regex_nfa.transitions, expected_transitions);
        assert_eq!(regex_nfa.accepting_state, expected_accepting_state);
    }

    #[test]
    fn nfa_to_dfa_test() {
        let input_nfa = Nfa {
            transitions: HashMap::from([
                ((0, None), vec![1, 7]),
                ((1, None), vec![2, 4]),
                ((2, Some('a')), vec![3]),
                ((3, None), vec![6]),
                ((4, Some('b')), vec![5]),
                ((5, None), vec![6]),
                ((6, None), vec![1, 7]),
            ]),
            accepting_state: 7,
        };

        let generated_dfa = nfa_to_dfa(&input_nfa);

        let expected_options = [
            HashMap::from([
                ((0, 'a'), 1),
                ((0, 'b'), 2),
                ((1, 'a'), 1),
                ((1, 'b'), 2),
                ((2, 'a'), 1),
                ((2, 'b'), 2),
            ]),
            HashMap::from([
                ((0, 'a'), 2),
                ((0, 'b'), 1),
                ((1, 'a'), 2),
                ((1, 'b'), 1),
                ((2, 'a'), 2),
                ((2, 'b'), 1),
            ]),
        ];
        let expected_accepting_states = HashSet::from([0, 1, 2]);

        assert!(
            expected_options.contains(&generated_dfa.transitions),
            "Transitions did not match any of the expected options."
        );
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }
}

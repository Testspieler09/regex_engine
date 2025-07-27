use crate::{Dfa, is_valid_regex, normalise_regex};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug, PartialEq)]
enum SymbolType {
    Normal,
    KleeneStar,
    Escaped,
}

#[derive(Debug)]
struct Nfa {
    transitions: HashMap<(u32, char), Vec<u32>>,
    accepting_states: HashSet<u32>,
}

#[derive(Debug)]
pub struct GlushkovDfa {
    transitions: HashMap<(u32, char), u32>,
    accepting_states: HashSet<u32>,
}

impl Dfa for GlushkovDfa {
    fn new(regex: &str) -> Self {
        if !is_valid_regex(regex) {
            panic!("{regex} is not a valid regular expression!");
        }

        let normalised_regex = normalise_regex(regex);
        let regex_nfa = glushkov_construction(&normalised_regex);
        dbg!(&regex_nfa);
        let mut regex_dfa = nfa_no_epsilon_to_dfa(&regex_nfa);
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

// GLUSHKOV CONSTRUCTION
fn glushkov_construction(regex: &str) -> Nfa {
    dbg!(&regex);
    let mut transitions: HashMap<(u32, char), Vec<u32>> = HashMap::new();
    let mut accepting_states: HashSet<u32> = HashSet::new();

    let states: HashMap<u32, (char, SymbolType, u32)> = index_states(regex);
    dbg!(&states);

    fill_sets(states, &mut accepting_states, &mut transitions);

    Nfa {
        transitions,
        accepting_states,
    }
}

fn index_states(regex: &str) -> HashMap<u32, (char, SymbolType, u32)> {
    let mut indexed_states: HashMap<u32, (char, SymbolType, u32)> = HashMap::new();
    let mut symbol_type: SymbolType = SymbolType::Normal;
    let mut group_stack: Vec<u32> = vec![0];
    let mut idx: u32 = 0;
    let mut next_group_id: u32 = 1;
    let mut chars = regex.chars().peekable();

    while let Some(symbol) = chars.next() {
        if symbol_type == SymbolType::Escaped {
            indexed_states.entry(idx).or_insert((
                symbol,
                symbol_type.clone(),
                *group_stack.last().unwrap(),
            ));
            idx += 1;
            symbol_type = SymbolType::Normal;
            continue;
        }

        match symbol {
            '|' => {
                // Start a new group for the next alternative
                let new_group_id = next_group_id;
                next_group_id += 1;
                // Replace the current group on the stack with the new one
                if let Some(last) = group_stack.last_mut() {
                    *last = new_group_id;
                }
            }
            '(' => {
                // Push the next group ID onto the stack for this grouping level
                let new_group_id = next_group_id;
                next_group_id += 1;
                group_stack.push(new_group_id);
            }
            ')' => {
                // Pop the current group and return to parent group
                group_stack.pop();
            }
            '*' => {
                symbol_type = SymbolType::Normal;
                continue;
            }
            '\\' => symbol_type = SymbolType::Escaped,
            _ => {
                if let Some(next_symbol) = chars.peek()
                    && matches!(*next_symbol, '*')
                {
                    symbol_type = SymbolType::KleeneStar
                }
                indexed_states.entry(idx).or_insert((
                    symbol,
                    symbol_type.clone(),
                    *group_stack.last().unwrap(),
                ));
                idx += 1;
            }
        }
    }
    indexed_states
}

fn fill_sets(
    states: HashMap<u32, (char, SymbolType, u32)>,
    accepting_states: &mut HashSet<u32>,
    transitions: &mut HashMap<(u32, char), Vec<u32>>,
) {
    let mut start_states = HashSet::new();

    let amount_states = states.len() as u32;
    if amount_states == 0 {
        return;
    }

    // Group states by their group index
    let mut groups: HashMap<u32, Vec<u32>> = HashMap::new();
    for (state_id, (_, _, group_idx)) in &states {
        groups.entry(*group_idx).or_default().push(*state_id);
    }

    // Sort states within each group
    for group in groups.values_mut() {
        group.sort();
    }

    // Determine start states (first state of each group)
    for group in groups.values() {
        if group.is_empty() {
            continue;
        }

        start_states.insert(group[0]);

        for i in 0..group.len() {
            let state = group[i];
            if let Some((_, symbol_type, _)) = states.get(&state) {
                if symbol_type == &SymbolType::KleeneStar && i + 1 < group.len() {
                    start_states.insert(group[i + 1]);
                }
            }
        }
    }

    // Build transitions and determine accepting states
    for (state_id, (symbol, symbol_type, group_idx)) in &states {
        let current_group = &groups[group_idx];
        let pos_in_group = current_group.iter().position(|&x| x == *state_id).unwrap();

        match symbol_type {
            SymbolType::Normal | SymbolType::Escaped => {
                if pos_in_group + 1 < current_group.len() {
                    let next_state = current_group[pos_in_group + 1];
                    transitions
                        .entry((*state_id, *symbol))
                        .or_default()
                        .push(next_state);
                } else {
                    accepting_states.insert(*state_id);
                }
            }
            SymbolType::KleeneStar => {
                transitions
                    .entry((*state_id, *symbol))
                    .or_default()
                    .push(*state_id);

                if pos_in_group + 1 < current_group.len() {
                    for next_state in current_group.iter().skip(pos_in_group + 1) {
                        transitions
                            .entry((*state_id, *symbol))
                            .or_default()
                            .push(*next_state);
                    }
                } else {
                    accepting_states.insert(*state_id);
                }
            }
        }
    }

    // Setup virtual (start-)state
    let virtual_start = states.keys().max().copied().unwrap_or(0) + 1;

    let symbol_to_first_state: Vec<(u32, char)> = start_states
        .iter()
        .map(|&s| (s, states.get(&s).expect("Expected an entry").0))
        .collect();

    for (first_state, symbol) in symbol_to_first_state {
        transitions
            .entry((virtual_start, symbol))
            .or_default()
            .push(first_state);
    }
}
// END GLUSHKOV CONSTRUCTION

fn nfa_no_epsilon_to_dfa(nfa: &Nfa) -> GlushkovDfa {
    let mut dfa_transitions = HashMap::new();
    let mut dfa_accepting_states = HashSet::new();

    // Map from sorted vector of NFA states to DFA state ID (for hashable key)
    let mut nfa_states_to_dfa_state: HashMap<Vec<u32>, u32> = HashMap::new();
    let mut next_dfa_state_id = 0u32;
    let mut work_queue = VecDeque::new();

    // Helper function to convert HashSet to sorted Vec for use as HashMap key
    let set_to_sorted_vec = |set: &HashSet<u32>| -> Vec<u32> {
        let mut vec: Vec<u32> = set.iter().cloned().collect();
        vec.sort_unstable();
        vec
    };

    // Get all possible input symbols from NFA transitions
    let alphabet: HashSet<char> = nfa.transitions.keys().map(|(_, symbol)| *symbol).collect();

    // Find all states that exist in the NFA
    let mut all_nfa_states = HashSet::new();
    for &(state, _) in nfa.transitions.keys() {
        all_nfa_states.insert(state);
    }
    for target_states in nfa.transitions.values() {
        for &state in target_states {
            all_nfa_states.insert(state);
        }
    }
    for &state in &nfa.accepting_states {
        all_nfa_states.insert(state);
    }

    // In a Glushkov NFA, state 0 is always the start state
    let start_state = 0;

    // Verify that state 0 exists in the NFA
    if !all_nfa_states.contains(&start_state) {
        panic!("Expected start state 0 not found in NFA states: {all_nfa_states:?}");
    }

    let start_state_set = {
        let mut set = HashSet::new();
        set.insert(start_state);
        set
    };

    // Create initial DFA state
    let start_dfa_state = next_dfa_state_id;
    next_dfa_state_id += 1;

    let start_state_key = set_to_sorted_vec(&start_state_set);
    nfa_states_to_dfa_state.insert(start_state_key, start_dfa_state);
    work_queue.push_back(start_state_set);

    // Process each DFA state
    while let Some(current_nfa_states) = work_queue.pop_front() {
        let current_state_key = set_to_sorted_vec(&current_nfa_states);
        let current_dfa_state = nfa_states_to_dfa_state[&current_state_key];

        // Check if this DFA state should be accepting
        if current_nfa_states
            .iter()
            .any(|&state| nfa.accepting_states.contains(&state))
        {
            dfa_accepting_states.insert(current_dfa_state);
        }

        // For each symbol in the alphabet
        for &symbol in &alphabet {
            let mut next_nfa_states = HashSet::new();

            // Collect all states reachable from current_nfa_states via symbol
            for &nfa_state in &current_nfa_states {
                if let Some(target_states) = nfa.transitions.get(&(nfa_state, symbol)) {
                    for &target_state in target_states {
                        next_nfa_states.insert(target_state);
                    }
                }
            }

            // Skip if no transitions exist for this symbol
            if next_nfa_states.is_empty() {
                continue;
            }

            // Get or create DFA state for this set of NFA states
            let next_state_key = set_to_sorted_vec(&next_nfa_states);
            let next_dfa_state =
                if let Some(&existing_state) = nfa_states_to_dfa_state.get(&next_state_key) {
                    existing_state
                } else {
                    let new_state = next_dfa_state_id;
                    next_dfa_state_id += 1;

                    nfa_states_to_dfa_state.insert(next_state_key.clone(), new_state);
                    work_queue.push_back(next_nfa_states);

                    new_state
                };

            // Add transition to DFA
            dfa_transitions.insert((current_dfa_state, symbol), next_dfa_state);
        }
    }

    GlushkovDfa {
        transitions: dfa_transitions,
        accepting_states: dfa_accepting_states,
    }
}

// fn nfa_no_epsilon_to_dfa(nfa: &Nfa) -> GlushkovDfa {
//     let mut dfa_transitions = HashMap::new();
//     let mut dfa_accepting_states = HashSet::new();
//
//     // Map from DFA state ID to the set of NFA states it represents
//     let mut dfa_state_to_nfa_states: HashMap<u32, HashSet<u32>> = HashMap::new();
//     // Map from sorted vector of NFA states to DFA state ID (for hashable key)
//     let mut nfa_states_to_dfa_state: HashMap<Vec<u32>, u32> = HashMap::new();
//
//     let mut next_dfa_state_id = 0u32;
//     let mut work_queue = VecDeque::new();
//
//     // Helper function to convert HashSet to sorted Vec for use as HashMap key
//     let set_to_sorted_vec = |set: &HashSet<u32>| -> Vec<u32> {
//         let mut vec: Vec<u32> = set.iter().cloned().collect();
//         vec.sort_unstable();
//         vec
//     };
//
//     // Get all possible input symbols from NFA transitions
//     let alphabet: HashSet<char> = nfa.transitions.keys().map(|(_, symbol)| *symbol).collect();
//
//     // Find the start state (assuming state 0 is the start state)
//     let start_state_set = {
//         let mut set = HashSet::new();
//         set.insert(0u32);
//         set
//     };
//
//     // Create initial DFA state
//     let start_dfa_state = next_dfa_state_id;
//     next_dfa_state_id += 1;
//
//     let start_state_key = set_to_sorted_vec(&start_state_set);
//     dfa_state_to_nfa_states.insert(start_dfa_state, start_state_set.clone());
//     nfa_states_to_dfa_state.insert(start_state_key, start_dfa_state);
//     work_queue.push_back(start_state_set);
//
//     // Process each DFA state
//     while let Some(current_nfa_states) = work_queue.pop_front() {
//         let current_state_key = set_to_sorted_vec(&current_nfa_states);
//         let current_dfa_state = nfa_states_to_dfa_state[&current_state_key];
//
//         // Check if this DFA state should be accepting
//         if current_nfa_states
//             .iter()
//             .any(|&state| nfa.accepting_states.contains(&state))
//         {
//             dfa_accepting_states.insert(current_dfa_state);
//         }
//
//         // For each symbol in the alphabet
//         for &symbol in &alphabet {
//             let mut next_nfa_states = HashSet::new();
//
//             // Collect all states reachable from current_nfa_states via symbol
//             for &nfa_state in &current_nfa_states {
//                 if let Some(target_states) = nfa.transitions.get(&(nfa_state, symbol)) {
//                     for &target_state in target_states {
//                         next_nfa_states.insert(target_state);
//                     }
//                 }
//             }
//
//             // Skip if no transitions exist for this symbol
//             if next_nfa_states.is_empty() {
//                 continue;
//             }
//
//             // Get or create DFA state for this set of NFA states
//             let next_state_key = set_to_sorted_vec(&next_nfa_states);
//             let next_dfa_state =
//                 if let Some(&existing_state) = nfa_states_to_dfa_state.get(&next_state_key) {
//                     existing_state
//                 } else {
//                     let new_state = next_dfa_state_id;
//                     next_dfa_state_id += 1;
//
//                     dfa_state_to_nfa_states.insert(new_state, next_nfa_states.clone());
//                     nfa_states_to_dfa_state.insert(next_state_key, new_state);
//                     work_queue.push_back(next_nfa_states);
//
//                     new_state
//                 };
//
//             // Add transition to DFA
//             dfa_transitions.insert((current_dfa_state, symbol), next_dfa_state);
//         }
//     }
//
//     GlushkovDfa {
//         transitions: dfa_transitions,
//         accepting_states: dfa_accepting_states,
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_single_character() {
        let expected = HashMap::from([(0, ('a', SymbolType::Normal, 0))]);

        let result = index_states("a");
        assert_eq!(result, expected, "Mismatch in single character test");
    }

    #[test]
    fn test_nfa_single_character() {
        let expected_finite = HashSet::from([0]);
        let expected_transitions: HashMap<(u32, char), Vec<u32>> =
            HashMap::from([((1, 'a'), vec![0])]);

        let result = glushkov_construction("a");
        assert_eq!(
            result.transitions, expected_transitions,
            "Mismatch in single character test"
        );
        assert_eq!(
            result.accepting_states, expected_finite,
            "Mismatch in single character test"
        );
    }

    #[test]
    fn test_nfa_single_character_kleene_star() {
        let expected_finite = HashSet::from([0]);
        let expected_transitions: HashMap<(u32, char), Vec<u32>> =
            HashMap::from([((0, 'a'), vec![0]), ((1, 'a'), vec![0])]);

        let result = glushkov_construction("a*");
        assert_eq!(
            result.transitions, expected_transitions,
            "Mismatch in single character test"
        );
        assert_eq!(
            result.accepting_states, expected_finite,
            "Mismatch in single character test"
        );
    }

    #[test]
    fn test_index_kleene_star() {
        let expected = HashMap::from([(0, ('a', SymbolType::KleeneStar, 0))]);

        let result = index_states("a*");
        assert_eq!(result, expected, "Mismatch in kleene star test");
    }

    #[test]
    fn test_index_union_and_groups() {
        let expected = HashMap::from([
            (0, ('a', SymbolType::Normal, 1)),
            (1, ('b', SymbolType::Normal, 2)),
        ]);

        let result = index_states("(a|b)");
        assert_eq!(result, expected, "Mismatch in union and groups test");
    }

    #[test]
    fn test_index_escaped_character() {
        let expected = HashMap::from([(0, ('a', SymbolType::Escaped, 0))]);

        let result = index_states("\\a");
        assert_eq!(result, expected, "Mismatch in escaped character test");
    }

    #[test]
    fn test_index_mixed_regex() {
        let expected = HashMap::from([
            (0, ('a', SymbolType::Normal, 0)),
            (1, ('*', SymbolType::Escaped, 0)),
            (2, ('b', SymbolType::Normal, 0)),
            (3, ('c', SymbolType::KleeneStar, 0)),
            (4, ('d', SymbolType::Normal, 0)),
            (5, ('e', SymbolType::Normal, 1)),
            (6, ('f', SymbolType::Normal, 2)),
            (7, ('g', SymbolType::Normal, 4)),
            (8, ('h', SymbolType::Normal, 5)),
            (9, ('i', SymbolType::Normal, 0)),
        ]);

        let result = index_states("a\\*bc*d(e|f|(g|h))i");
        assert_eq!(result, expected, "Mismatch in mixed regex test");
    }

    #[test]
    fn test_index_too_many_brackets() {
        let expected = HashMap::from([
            (0, ('a', SymbolType::KleeneStar, 0)),
            (1, ('b', SymbolType::Normal, 0)),
            (2, ('c', SymbolType::Normal, 3)),
            (3, ('d', SymbolType::Normal, 4)),
            (4, ('e', SymbolType::Normal, 5)),
            (5, ('f', SymbolType::Normal, 5)),
        ]);

        let result = index_states("a*b|((c|d))|ef");
        assert_eq!(result, expected, "Mismatch in mixed regex test");
    }

    #[test]
    fn test_fill_sets_too_many_brackets() {
        let states = index_states("a*b|(c|d)|ef");
        let mut finite_states: HashSet<u32> = HashSet::new();
        let mut transitions: HashMap<(u32, char), Vec<u32>> = HashMap::new();

        let expected_finite_set: HashSet<u32> = HashSet::from([1, 2, 3, 5]);
        let expected_transitions: HashMap<(u32, char), Vec<u32>> = HashMap::from([
            ((6, 'a'), vec![0]),
            ((6, 'b'), vec![1]),
            ((6, 'c'), vec![2]),
            ((6, 'd'), vec![3]),
            ((6, 'e'), vec![4]),
            ((0, 'a'), vec![0, 1]),
            ((4, 'e'), vec![5]),
        ]);

        fill_sets(states, &mut finite_states, &mut transitions);

        assert_eq!(finite_states, expected_finite_set);
        assert_eq!(transitions, expected_transitions);
    }

    #[test]
    fn test_fill_sets_complex() {
        let states = index_states("a*b*c|d*e");
        let mut finite_states: HashSet<u32> = HashSet::new();
        let mut transitions: HashMap<(u32, char), Vec<u32>> = HashMap::new();

        let expected_finite_set: HashSet<u32> = HashSet::from([2, 4]);
        let expected_transitions = HashMap::from([
            ((5, 'a'), vec![0]),
            ((5, 'b'), vec![1]),
            ((5, 'c'), vec![2]),
            ((5, 'd'), vec![3]),
            ((5, 'e'), vec![4]),
            ((0, 'a'), vec![0, 1, 2]),
            ((1, 'b'), vec![1, 2]),
            ((3, 'd'), vec![3, 4]),
        ]);

        fill_sets(states, &mut finite_states, &mut transitions);

        assert_eq!(finite_states, expected_finite_set);
        assert_eq!(transitions, expected_transitions);
    }

    #[test]
    fn nfa_to_dfa_simple_test() {
        // NFA that accepts exactly "a"
        // State 0 --a--> State 1 (accepting)
        let input_nfa = Nfa {
            transitions: HashMap::from([((0, 'a'), vec![1])]),
            accepting_states: HashSet::from([1]),
        };

        let generated_dfa = nfa_no_epsilon_to_dfa(&input_nfa);

        let expected_transitions = HashMap::from([((0, 'a'), 1)]);
        let expected_accepting_states = HashSet::from([1]);

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }

    #[test]
    fn nfa_to_dfa_sequence_test() {
        // NFA that accepts exactly "ab"
        // State 0 --a--> State 1 --b--> State 2 (accepting)
        let input_nfa = Nfa {
            transitions: HashMap::from([((0, 'a'), vec![1]), ((1, 'b'), vec![2])]),
            accepting_states: HashSet::from([2]),
        };

        let generated_dfa = nfa_no_epsilon_to_dfa(&input_nfa);

        let expected_transitions = HashMap::from([((0, 'a'), 1), ((1, 'b'), 2)]);
        let expected_accepting_states = HashSet::from([2]);

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }

    #[test]
    fn nfa_to_dfa_alternation_test() {
        // NFA that accepts "a" or "b"
        // State 0 --a--> State 1 (accepting)
        // State 0 --b--> State 2 (accepting)
        let input_nfa = Nfa {
            transitions: HashMap::from([((0, 'a'), vec![1]), ((0, 'b'), vec![2])]),
            accepting_states: HashSet::from([1, 2]),
        };

        let generated_dfa = nfa_no_epsilon_to_dfa(&input_nfa);

        let expected_transitions = [
            HashMap::from([((0, 'a'), 1), ((0, 'b'), 2)]),
            HashMap::from([((0, 'a'), 2), ((0, 'b'), 1)]),
        ];
        let expected_accepting_states = HashSet::from([1, 2]);

        assert!(
            generated_dfa.transitions == expected_transitions[0]
                || generated_dfa.transitions == expected_transitions[1],
            "generated_dfa.transitions did not match either expected set"
        );
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }

    #[test]
    fn nfa_to_dfa_nondeterministic_test() {
        // NFA with nondeterministic transition
        // State 0 --a--> State 1, State 2
        // State 1 --b--> State 3 (accepting)
        // State 2 --c--> State 3 (accepting)
        let input_nfa = Nfa {
            transitions: HashMap::from([
                ((0, 'a'), vec![1, 2]),
                ((1, 'b'), vec![3]),
                ((2, 'c'), vec![3]),
            ]),
            accepting_states: HashSet::from([3]),
        };

        let generated_dfa = nfa_no_epsilon_to_dfa(&input_nfa);

        // After 'a' from state 0, we should be in a state representing {1, 2}
        // Let's call this combined state "1" in the DFA
        let expected_transitions = HashMap::from([
            ((0, 'a'), 1), // {0} --a--> {1,2} (DFA state 1)
            ((1, 'b'), 2), // {1,2} --b--> {3} (DFA state 2)
            ((1, 'c'), 2), // {1,2} --c--> {3} (DFA state 2)
        ]);
        let expected_accepting_states = HashSet::from([2]); // DFA state 2 represents {3}

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }

    #[test]
    fn nfa_to_dfa_multiple_accepting_test() {
        // NFA where multiple paths lead to accepting states
        // State 0 --a--> State 1 (accepting)
        // State 0 --a--> State 2 --b--> State 3 (accepting)
        let input_nfa = Nfa {
            transitions: HashMap::from([((0, 'a'), vec![1, 2]), ((2, 'b'), vec![3])]),
            accepting_states: HashSet::from([1, 3]),
        };

        let generated_dfa = nfa_no_epsilon_to_dfa(&input_nfa);

        // After 'a' from state 0, we're in state representing {1, 2}
        // This should be accepting because it contains state 1
        let expected_transitions = HashMap::from([
            ((0, 'a'), 1), // {0} --a--> {1,2} (DFA state 1)
            ((1, 'b'), 2), // {1,2} --b--> {3} (DFA state 2)
        ]);
        let expected_accepting_states = HashSet::from([1, 2]); // Both DFA states are accepting

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }

    #[test]
    fn nfa_to_dfa_self_loop_test() {
        // NFA with self-loop: accepts a*
        // State 0 (accepting) --a--> State 0
        let input_nfa = Nfa {
            transitions: HashMap::from([((0, 'a'), vec![0])]),
            accepting_states: HashSet::from([0]),
        };

        let generated_dfa = nfa_no_epsilon_to_dfa(&input_nfa);

        let expected_transitions = HashMap::from([
            ((0, 'a'), 0), // Self-loop
        ]);
        let expected_accepting_states = HashSet::from([0]);

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);
    }
}

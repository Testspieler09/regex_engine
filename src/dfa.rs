use std::collections::{HashMap, HashSet};

#[derive(debug)]
struct NFA {
    states: HashSet<u32>,
    alphabet: HashSet<char>,
    transitions: HashMap<(u32, char), Vec<u32>>,
    start_state: u32,
    accepting_states: HashSet<u32>,
}

#[derive(debug)]
struct DFA {
    states: HashSet<u32>,
    alphabet: HashSet<char>,
    transitions: HashMap<(u32, char), u32>,
    start_state: u32,
    accepting_states: HashSet<u32>,
}

fn thompson_construction(regex: &str) -> NFA {
    // TODO: implement this
    NFA {
        states,
        alphabet,
        transitions,
        start_state,
        accepting_states,
    }
}

fn nfa_to_dfa(regex_nfa: &NFA) -> DFA {
    // TODO: implement this
    DFA {
        states,
        alphabet,
        transitions,
        start_state,
        accepting_states,
    }
}

impl DFA {
    fn new(regex: &str) -> Self {
        // TODO: Implement Thompson construction
        let regex_nfa: NFA = thompson_construction(regex);
        // TODO: Converting NFA to DFA
        nfa_to_dfa(regex_nfa)
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

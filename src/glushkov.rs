use crate::regex_engine::{is_valid_regex, normalise_regex};
use std::collections::{HashMap, HashSet};

struct NFA {
    transitions: HashMap<(u32, Option<char>), Vec<u32>>,
    accepting_state: u32,
}

pub struct DFA {
    transitions: HashMap<(u32, Option<char>), u32>,
    accepting_states: HashSet<u32>,
}

// GLUSHKOV CONSTRUCTION
fn glushkov_construction(regex: &str) -> NFA {
    // TODO: Step 1 (rename letters / index them)
    // TODO: Step 2a ()
    // TODO: Step 2b ()
    // TODO: Step 3 ()
    // TODO: Step 4 ()
    todo!()
}

fn nfa_no_epsilon_to_dfa() {
    todo!()
}
// END GLUSHKOV CONSTRUCTION

impl DFA {
    pub fn new(regex: &str) -> Self {
        if !is_valid_regex(regex) {
            panic!("{} is not a valid regular expression!", regex);
        }

        let normalised_regex = normalise_regex(&regex);
        todo!()
    }

    pub fn process(&self, input: &str) -> bool {
        todo!()
    }

    pub fn find_first_match<'a>(&self, text: &'a str) -> Option<&'a str> {
        todo!()
    }

    pub fn find_all_matches<'a>(&self, input: &'a str) -> Vec<&'a str> {
        todo!()
    }
}

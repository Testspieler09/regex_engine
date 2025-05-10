use crate::regex_engine::{is_valid_regex, normalise_regex};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
enum SymbolType {
    Normal,
    KleeneStar,
    Escaped,
}

struct NFA {
    transitions: HashMap<(u32, char), Vec<u32>>,
    accepting_states: HashSet<u32>,
}

pub struct DFA {
    transitions: HashMap<(u32, char), u32>,
    accepting_states: HashSet<u32>,
}

// GLUSHKOV CONSTRUCTION
fn glushkov_construction(regex: &str) -> NFA {
    let mut transitions: HashMap<(u32, char), Vec<u32>> = HashMap::new();
    let mut accepting_states: HashSet<u32> = HashSet::new();

    let states: HashMap<(u32, char), (SymbolType, u32)> = index_states(regex);

    // TODO: Construct transitions and accepting states using position_index
    NFA {
        transitions,
        accepting_states,
    }
}

fn index_states(regex: &str) -> HashMap<(u32, char), (SymbolType, u32)> {
    let mut indexed_states: HashMap<(u32, char), (SymbolType, u32)> = HashMap::new();
    let mut symbol_type: SymbolType = SymbolType::Normal;
    let mut union_count: Vec<u32> = vec![0];
    let mut idx: u32 = 0;
    let mut group_index: u32 = 0;

    let mut chars = regex.chars().peekable();

    while let Some(symbol) = chars.next() {
        if symbol_type == SymbolType::Escaped {
            indexed_states
                .entry((idx as u32, symbol))
                .or_insert((symbol_type.clone(), group_index));

            idx += 1;
            symbol_type = SymbolType::Normal;
            continue;
        }

        match symbol {
            '|' => {
                if let Some(last_element) = union_count.last_mut() {
                    *last_element += 1;
                }
                group_index += 1;
            }
            '(' => {
                union_count.push(0);
                group_index += 1;
            }
            ')' => {
                group_index -= union_count.pop().unwrap() + 1;
            }
            '*' => {
                symbol_type = SymbolType::Normal;
                continue;
            }
            '\\' => symbol_type = SymbolType::Escaped,
            _ => {
                if let Some(next_symbol) = chars.peek() {
                    if *next_symbol == '*' {
                        symbol_type = SymbolType::KleeneStar
                    }
                }

                indexed_states
                    .entry((idx as u32, symbol))
                    .or_insert((symbol_type.clone(), group_index));

                idx += 1;
            }
        }
    }

    indexed_states
}
// END GLUSHKOV CONSTRUCTION

fn nfa_no_epsilon_to_dfa(nfa: &NFA) -> DFA {
    todo!()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character() {
        let expected = HashMap::from([((0, 'a'), (SymbolType::Normal, 0))]);

        let result = index_states("a");
        assert_eq!(result, expected, "Mismatch in single character test");
    }

    #[test]
    fn test_kleene_star() {
        let expected = HashMap::from([((0, 'a'), (SymbolType::KleeneStar, 0))]);

        let result = index_states("a*");
        assert_eq!(result, expected, "Mismatch in kleene star test");
    }

    #[test]
    fn test_union_and_groups() {
        let expected = HashMap::from([
            ((0, 'a'), (SymbolType::Normal, 1)),
            ((1, 'b'), (SymbolType::Normal, 2)),
        ]);

        let result = index_states("(a|b)");
        assert_eq!(result, expected, "Mismatch in union and groups test");
    }

    #[test]
    fn test_escaped_character() {
        let expected = HashMap::from([((0, 'a'), (SymbolType::Escaped, 0))]);

        let result = index_states("\\a");
        assert_eq!(result, expected, "Mismatch in escaped character test");
    }

    #[test]
    fn test_mixed_regex() {
        let expected = HashMap::from([
            ((0, 'a'), (SymbolType::Normal, 0)),
            ((1, '*'), (SymbolType::Escaped, 0)),
            ((2, 'b'), (SymbolType::Normal, 0)),
            ((3, 'c'), (SymbolType::KleeneStar, 0)),
            ((4, 'd'), (SymbolType::Normal, 0)),
            ((5, 'e'), (SymbolType::Normal, 1)),
            ((6, 'f'), (SymbolType::Normal, 2)),
            ((7, 'g'), (SymbolType::Normal, 4)),
            ((8, 'h'), (SymbolType::Normal, 5)),
            ((9, 'i'), (SymbolType::Normal, 0)),
        ]);

        let result = index_states("a\\*bc*d(e|f|(g|h))i");
        assert_eq!(result, expected, "Mismatch in mixed regex test");
    }
}

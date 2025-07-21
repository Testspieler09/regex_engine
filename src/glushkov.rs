use crate::{Dfa, is_valid_regex, normalise_regex};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
enum SymbolType {
    Normal,
    KleeneStar,
    Escaped,
}

struct Nfa {
    transitions: HashMap<(u32, char), Vec<u32>>,
    accepting_states: HashSet<u32>,
}

pub struct GlushkovDfa {
    transitions: HashMap<(u32, char), u32>,
    accepting_states: HashSet<u32>,
}

impl Dfa for GlushkovDfa {
    fn new(regex: &str) -> Self {
        if !is_valid_regex(regex) {
            panic!("{regex} is not a valid regular expression!");
        }

        let normalised_regex = normalise_regex(&regex);
        todo!()
    }

    fn get_transitions(&self) -> &HashMap<(u32, char), u32> {
        &self.transitions
    }

    fn get_accepting_states(&self) -> &HashSet<u32> {
        &self.accepting_states
    }
}

// GLUSHKOV CONSTRUCTION
fn glushkov_construction(regex: &str) -> Nfa {
    let mut transitions: HashMap<(u32, char), Vec<u32>> = HashMap::new();
    let mut accepting_states: HashSet<u32> = HashSet::new();

    let states: HashMap<u32, (char, SymbolType, u32)> = index_states(regex);

    let mut start_states: HashSet<u32> = HashSet::new();

    fill_sets(
        states,
        &mut start_states,
        &mut accepting_states,
        &mut transitions,
    );

    // TODO: Construct transitions and accepting states using position_index
    Nfa {
        transitions,
        accepting_states,
    }
}

fn index_states(regex: &str) -> HashMap<u32, (char, SymbolType, u32)> {
    let mut indexed_states: HashMap<u32, (char, SymbolType, u32)> = HashMap::new();
    let mut symbol_type = SymbolType::Normal;
    let mut union_count: Vec<u32> = vec![0];
    let mut idx: u32 = 0;
    let mut group_index: u32 = 0;

    // New stack to track if a group is meaningful
    let mut group_stack: Vec<Option<u32>> = vec![]; // Some(index) if real, None if ignored

    let mut chars = regex.chars().peekable();

    while let Some(symbol) = chars.next() {
        if symbol_type == SymbolType::Escaped {
            indexed_states.insert(idx, (symbol, symbol_type.clone(), group_index));
            idx += 1;
            symbol_type = SymbolType::Normal;
            continue;
        }

        match symbol {
            '|' => {
                if let Some(last_union) = union_count.last_mut() {
                    *last_union += 1;
                }
                if let Some(Some(_)) = group_stack.last_mut() {
                    // still a real group, do nothing here
                } else if let Some(group) = group_stack.last_mut() {
                    // this group is now meaningful, assign it an index
                    *group = Some(group_index);
                    group_index += 1;
                }
            }
            '(' => {
                union_count.push(0);
                group_stack.push(None); // not yet known if meaningful
            }
            ')' => {
                union_count.pop();

                match group_stack.pop() {
                    Some(Some(_)) => {
                        // it was meaningful, nothing to change
                    }
                    Some(None) => {
                        // the group was never promoted to real => do nothing
                    }
                    None => panic!("Mismatched parentheses"),
                }
            }
            '*' => {
                symbol_type = SymbolType::Normal;
                continue;
            }
            '\\' => symbol_type = SymbolType::Escaped,
            _ => {
                if let Some(next) = chars.peek() {
                    if *next == '*' {
                        symbol_type = SymbolType::KleeneStar;
                    }
                }

                // if we're inside a group that hasn't been assigned an index yet, assign now
                if let Some(group) = group_stack.last_mut() {
                    if group.is_none() {
                        *group = Some(group_index);
                        group_index += 1;
                    }
                }

                // get current group idx for this symbol
                let current_group = group_stack.last().and_then(|g| *g).unwrap_or(group_index);

                indexed_states.insert(idx, (symbol, symbol_type.clone(), current_group));
                idx += 1;
            }
        }
    }

    // let mut indexed_states: HashMap<u32, (char, SymbolType, u32)> = HashMap::new();
    // let mut symbol_type: SymbolType = SymbolType::Normal;
    // let mut union_count: Vec<u32> = vec![0];
    // let mut idx: u32 = 0;
    // let mut group_index: u32 = 0;
    //
    // let mut chars = regex.chars().peekable();
    //
    // while let Some(symbol) = chars.next() {
    //     if symbol_type == SymbolType::Escaped {
    //         indexed_states
    //             .entry(idx)
    //             .or_insert((symbol, symbol_type.clone(), group_index));
    //
    //         idx += 1;
    //         symbol_type = SymbolType::Normal;
    //         continue;
    //     }
    //
    //     println!("{union_count:?}, {symbol:?}");
    //     match symbol {
    //         '|' => {
    //             if let Some(last_element) = union_count.last_mut() {
    //                 *last_element += 1;
    //             }
    //             group_index += 1;
    //         }
    //         // FIX: the paranthasis are not working correctly e.g. x|(x|y)|x <=> x|x|y|x
    //         '(' => {
    //             union_count.push(0);
    //             group_index += 1;
    //         }
    //         ')' => {
    //             let unions_last_grouping = union_count.pop().unwrap();
    //             if unions_last_grouping == 0 {
    //                 continue;
    //             }
    //             group_index -= unions_last_grouping + 1;
    //         }
    //         '*' => {
    //             symbol_type = SymbolType::Normal;
    //             continue;
    //         }
    //         '\\' => symbol_type = SymbolType::Escaped,
    //         _ => {
    //             if let Some(next_symbol) = chars.peek() {
    //                 if *next_symbol == '*' {
    //                     symbol_type = SymbolType::KleeneStar
    //                 }
    //             }
    //
    //             indexed_states
    //                 .entry(idx)
    //                 .or_insert((symbol, symbol_type.clone(), group_index));
    //
    //             idx += 1;
    //         }
    //     }
    // }

    indexed_states
}

fn fill_sets(
    states: HashMap<u32, (char, SymbolType, u32)>,
    start_states: &mut HashSet<u32>,
    finite_states: &mut HashSet<u32>,
    tranisitions: &mut HashMap<(u32, char), Vec<u32>>,
) {
    let mut idx: u32 = 1;
    let amount_states: u32 = states.len() as u32;

    if amount_states == 0 {
        return;
    }

    // tranisitions
    //     .entry((amount_states, states[&0].0))
    //     .or_insert(vec![0]);
    start_states.insert(0);

    let mut last_symbol_type: &SymbolType = &states[&0].1;
    let mut last_group_idx: u32 = 0;
    let mut check_next_group: bool = false; // NOTE: can also be thought of as group_is_exhausted

    loop {
        let (_symbol, symbol_type, group_idx) = &states[&idx];
        // Skip forwards to next group
        if check_next_group {
            if *group_idx != last_group_idx {
                start_states.insert(idx);
                last_symbol_type = symbol_type;
                last_group_idx = *group_idx;
                check_next_group = false;
                // continue;
            }

            if idx < amount_states - 1 {
                idx += 1;
                continue;
            } else {
                break;
            }
        }

        if *group_idx != last_group_idx {
            start_states.insert(idx);
            last_group_idx = *group_idx;
            last_symbol_type = symbol_type;
            check_next_group = true;

            if idx < amount_states - 1 {
                idx += 1;
                continue;
            } else {
                break;
            }
        }

        match last_symbol_type {
            SymbolType::Normal | SymbolType::Escaped => {
                check_next_group = true;
            }
            SymbolType::KleeneStar => {
                start_states.insert(idx);
                check_next_group = false;
            }
        }

        last_symbol_type = symbol_type;

        if idx < amount_states - 1 {
            idx += 1;
        } else {
            break;
        }
    }
}
// END GLUSHKOV CONSTRUCTION

fn nfa_no_epsilon_to_dfa(nfa: &Nfa) -> GlushkovDfa {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character() {
        let expected = HashMap::from([(0, ('a', SymbolType::Normal, 0))]);

        let result = index_states("a");
        assert_eq!(result, expected, "Mismatch in single character test");
    }

    #[test]
    fn test_kleene_star() {
        let expected = HashMap::from([(0, ('a', SymbolType::KleeneStar, 0))]);

        let result = index_states("a*");
        assert_eq!(result, expected, "Mismatch in kleene star test");
    }

    #[test]
    fn test_union_and_groups() {
        let expected = HashMap::from([
            (0, ('a', SymbolType::Normal, 1)),
            (1, ('b', SymbolType::Normal, 2)),
        ]);

        let result = index_states("(a|b)");
        assert_eq!(result, expected, "Mismatch in union and groups test");
    }

    #[test]
    fn test_escaped_character() {
        let expected = HashMap::from([(0, ('a', SymbolType::Escaped, 0))]);

        let result = index_states("\\a");
        assert_eq!(result, expected, "Mismatch in escaped character test");
    }

    #[test]
    fn test_mixed_regex() {
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
    fn test_too_many_brackets() {
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
    fn test_fill_sets() {
        let states = index_states("a*b|(c|d)|ef");
        let mut start_states: HashSet<u32> = HashSet::new();
        let mut finite_states: HashSet<u32> = HashSet::new();
        let mut transitions: HashMap<(u32, char), Vec<u32>> = HashMap::new();

        let expected_start_set: HashSet<u32> = HashSet::from([0, 1, 2, 3, 4]);
        let expected_finite_set: HashSet<u32> = HashSet::new();
        let expected_transions: HashMap<(u32, char), Vec<u32>> = HashMap::new();

        fill_sets(
            states,
            &mut start_states,
            &mut finite_states,
            &mut transitions,
        );

        assert_eq!(start_states, expected_start_set);
        assert_eq!(finite_states, expected_finite_set);
        assert_eq!(transitions, expected_transions);
    }
}

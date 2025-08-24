use crate::{Dfa, is_valid_regex, normalise_regex};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
enum RegexAst {
    Char(char),
    Concat(Vec<RegexAst>),
    Alternation(Vec<RegexAst>),
    KleeneStar(Box<RegexAst>),
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
    fn new(regex: &str) -> Result<Self, String> {
        if !is_valid_regex(regex) {
            return Err(format!("{regex} is not a valid regular expression!"));
        }

        let normalised_regex = normalise_regex(regex);
        let ast = parse_regex(&normalised_regex)?;
        let nfa = glushkov_construction(ast)?;
        let mut regex_dfa = nfa_to_dfa(nfa);

        <Self as Dfa>::optimise_dfa(&mut regex_dfa);
        Ok(regex_dfa)
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

// Parser for regex string to AST
fn parse_regex(regex: &str) -> Result<RegexAst, String> {
    let chars: Vec<char> = regex.chars().collect();
    let (ast, pos) = parse_alternation(&chars, 0)?;

    if pos != chars.len() {
        return Err("Unexpected characters at end of regex".to_string());
    }

    Ok(ast)
}

fn parse_alternation(chars: &[char], mut pos: usize) -> Result<(RegexAst, usize), String> {
    let mut alternatives = Vec::new();

    let (first_alt, new_pos) = parse_concatenation(chars, pos)?;
    alternatives.push(first_alt);
    pos = new_pos;

    while pos < chars.len() && chars[pos] == '|' {
        pos += 1; // skip '|'
        let (alt, new_pos) = parse_concatenation(chars, pos)?;
        alternatives.push(alt);
        pos = new_pos;
    }

    if alternatives.len() == 1 {
        Ok((alternatives.into_iter().next().unwrap(), pos))
    } else {
        Ok((RegexAst::Alternation(alternatives), pos))
    }
}

fn parse_concatenation(chars: &[char], mut pos: usize) -> Result<(RegexAst, usize), String> {
    let mut elements = Vec::new();

    while pos < chars.len() && chars[pos] != '|' && chars[pos] != ')' {
        let (element, new_pos) = parse_factor(chars, pos)?;
        elements.push(element);
        pos = new_pos;
    }

    // Handle empty concatenation (empty alternative)
    if elements.is_empty() {
        // Return an epsilon (empty string) represented as an empty concatenation
        return Ok((RegexAst::Concat(vec![]), pos));
    }

    if elements.len() == 1 {
        Ok((elements.into_iter().next().unwrap(), pos))
    } else {
        Ok((RegexAst::Concat(elements), pos))
    }
}

fn parse_factor(chars: &[char], mut pos: usize) -> Result<(RegexAst, usize), String> {
    if pos >= chars.len() {
        return Err("Unexpected end of regex".to_string());
    }

    let (base, new_pos) = match chars[pos] {
        '(' => {
            pos += 1; // skip '('
            let (inner, inner_pos) = parse_alternation(chars, pos)?;
            if inner_pos >= chars.len() || chars[inner_pos] != ')' {
                return Err("Unmatched opening parenthesis".to_string());
            }
            (inner, inner_pos + 1) // skip ')'
        }
        '\\' => {
            if pos + 1 >= chars.len() {
                return Err("Invalid escape sequence".to_string());
            }
            pos += 1; // skip '\'
            (RegexAst::Char(chars[pos]), pos + 1)
        }
        c if c.is_ascii() && !"()|*+\\".contains(c) => (RegexAst::Char(c), pos + 1),
        _ => {
            return Err(format!("Unexpected character: {}", chars[pos]));
        }
    };

    pos = new_pos;

    // Check for Kleene star
    if pos < chars.len() && chars[pos] == '*' {
        pos += 1;
        Ok((RegexAst::KleeneStar(Box::new(base)), pos))
    } else {
        Ok((base, pos))
    }
}

fn glushkov_construction(ast: RegexAst) -> Result<Nfa, String> {
    let mut state_counter = 0u32;
    let mut state_to_char: HashMap<u32, char> = HashMap::new();

    // Assign unique state numbers to each character occurrence
    assign_positions(&ast, &mut state_counter, &mut state_to_char);

    let start_state = state_counter;

    // Compute First, Last, Follow sets - each with fresh position counter
    let first_set = first(&ast);
    let last_set = last(&ast);
    let follow_map = follow(&ast);

    // Build NFA
    let mut transitions = HashMap::new();
    let mut accepting_states = HashSet::new();

    // Transitions from start state
    for &state in &first_set {
        if let Some(&ch) = state_to_char.get(&state) {
            transitions
                .entry((start_state, ch))
                .or_insert_with(Vec::new)
                .push(state);
        }
    }

    // Internal transitions based on follow sets
    for (state, follow_states) in follow_map {
        for &follow_state in &follow_states {
            if let Some(&ch) = state_to_char.get(&follow_state) {
                transitions
                    .entry((state, ch))
                    .or_insert_with(Vec::new)
                    .push(follow_state);
            }
        }
    }

    // Accepting states
    if nullable(&ast) {
        accepting_states.insert(start_state);
    }
    for &state in &last_set {
        accepting_states.insert(state);
    }

    Ok(Nfa {
        transitions,
        accepting_states,
    })
}

fn first(ast: &RegexAst) -> HashSet<u32> {
    let mut positions = HashMap::new();
    let mut counter = 0;
    map_ast_to_positions(ast, &mut counter, &mut positions);
    first_positions(ast, &positions)
}

fn last(ast: &RegexAst) -> HashSet<u32> {
    let mut positions = HashMap::new();
    let mut counter = 0;
    map_ast_to_positions(ast, &mut counter, &mut positions);
    last_positions(ast, &positions)
}

fn follow(ast: &RegexAst) -> HashMap<u32, HashSet<u32>> {
    let mut positions = HashMap::new();
    let mut counter = 0;
    map_ast_to_positions(ast, &mut counter, &mut positions);

    let mut result = HashMap::new();
    follow_positions(ast, &positions, &mut result);
    result
}

// Helper function to create a mapping from AST nodes to their position ranges
fn map_ast_to_positions(
    ast: &RegexAst,
    counter: &mut u32,
    positions: &mut HashMap<*const RegexAst, (u32, u32)>,
) {
    let start_pos = *counter;

    match ast {
        RegexAst::Char(_) => {
            *counter += 1;
        }
        RegexAst::Concat(elements) => {
            for element in elements {
                map_ast_to_positions(element, counter, positions);
            }
        }
        RegexAst::Alternation(alternatives) => {
            for alt in alternatives {
                map_ast_to_positions(alt, counter, positions);
            }
        }
        RegexAst::KleeneStar(inner) => {
            map_ast_to_positions(inner, counter, positions);
        }
    }

    positions.insert(ast as *const RegexAst, (start_pos, *counter));
}

fn first_positions(
    ast: &RegexAst,
    positions: &HashMap<*const RegexAst, (u32, u32)>,
) -> HashSet<u32> {
    match ast {
        RegexAst::Char(_) => {
            let (start_pos, _) = positions[&(ast as *const RegexAst)];
            let mut result = HashSet::new();
            result.insert(start_pos);
            result
        }
        RegexAst::Concat(elements) => {
            let mut result = HashSet::new();
            for element in elements {
                result.extend(first_positions(element, positions));
                if !nullable(element) {
                    break;
                }
            }
            result
        }
        RegexAst::Alternation(alternatives) => {
            let mut result = HashSet::new();
            for alt in alternatives {
                result.extend(first_positions(alt, positions));
            }
            result
        }
        RegexAst::KleeneStar(inner) => first_positions(inner, positions),
    }
}

fn last_positions(
    ast: &RegexAst,
    positions: &HashMap<*const RegexAst, (u32, u32)>,
) -> HashSet<u32> {
    match ast {
        RegexAst::Char(_) => {
            let (start_pos, _) = positions[&(ast as *const RegexAst)];
            let mut result = HashSet::new();
            result.insert(start_pos);
            result
        }
        RegexAst::Concat(elements) => {
            let mut result = HashSet::new();
            for element in elements.iter().rev() {
                result.extend(last_positions(element, positions));
                if !nullable(element) {
                    break;
                }
            }
            result
        }
        RegexAst::Alternation(alternatives) => {
            let mut result = HashSet::new();
            for alt in alternatives {
                result.extend(last_positions(alt, positions));
            }
            result
        }
        RegexAst::KleeneStar(inner) => last_positions(inner, positions),
    }
}

fn follow_positions(
    ast: &RegexAst,
    positions: &HashMap<*const RegexAst, (u32, u32)>,
    result: &mut HashMap<u32, HashSet<u32>>,
) {
    match ast {
        RegexAst::Char(_) => {
            // Base case - no follow computation needed
        }
        RegexAst::Concat(elements) => {
            // Process each element recursively
            for element in elements {
                follow_positions(element, positions, result);
            }

            // Add follow relationships between consecutive elements
            for i in 0..elements.len() {
                let last_i = last_positions(&elements[i], positions);

                // For each subsequent element j > i
                for j in (i + 1)..elements.len() {
                    // Check if all elements between i and j are nullable
                    let all_between_nullable = elements[(i + 1)..j].iter().all(nullable);

                    if j == i + 1 || all_between_nullable {
                        let first_j = first_positions(&elements[j], positions);

                        // Add follow relationships from last(i) to first(j)
                        for &last_state in &last_i {
                            result.entry(last_state).or_default().extend(&first_j);
                        }
                    }

                    // If element j is not nullable, we can't skip further
                    if !nullable(&elements[j]) {
                        break;
                    }
                }
            }
        }
        RegexAst::Alternation(alternatives) => {
            for alt in alternatives {
                follow_positions(alt, positions, result);
            }
        }
        RegexAst::KleeneStar(inner) => {
            follow_positions(inner, positions, result);

            // Kleene star: last positions can loop back to first positions
            let inner_last = last_positions(inner, positions);
            let inner_first = first_positions(inner, positions);

            for &last_state in &inner_last {
                result.entry(last_state).or_default().extend(&inner_first);
            }
        }
    }
}

fn nullable(ast: &RegexAst) -> bool {
    match ast {
        RegexAst::Char(_) => false,
        RegexAst::Concat(elements) => {
            // Empty concat is nullable (represents epsilon)
            elements.is_empty() || elements.iter().all(nullable)
        }
        RegexAst::Alternation(alternatives) => alternatives.iter().any(nullable),
        RegexAst::KleeneStar(_) => true,
    }
}

fn assign_positions(ast: &RegexAst, counter: &mut u32, state_to_char: &mut HashMap<u32, char>) {
    match ast {
        RegexAst::Char(ch) => {
            let state = *counter;
            *counter += 1;
            state_to_char.insert(state, *ch);
        }
        RegexAst::Concat(elements) => {
            for element in elements {
                assign_positions(element, counter, state_to_char);
            }
        }
        RegexAst::Alternation(alternatives) => {
            for alt in alternatives {
                assign_positions(alt, counter, state_to_char);
            }
        }
        RegexAst::KleeneStar(inner) => {
            assign_positions(inner, counter, state_to_char);
        }
    }
}

fn nfa_to_dfa(nfa: Nfa) -> GlushkovDfa {
    let mut dfa_transitions = HashMap::new();
    let mut dfa_accepting_states = HashSet::new();
    let mut state_sets_to_dfa_state: HashMap<BTreeSet<u32>, u32> = HashMap::new();
    let mut queue = VecDeque::new();
    let mut next_dfa_state = 0u32;

    // Get alphabet from NFA
    let alphabet: HashSet<char> = nfa.transitions.keys().map(|(_, ch)| *ch).collect();

    // Find start state (highest numbered state in NFA)
    let mut all_nfa_states = HashSet::new();

    for &(from_state, _) in nfa.transitions.keys() {
        all_nfa_states.insert(from_state);
    }
    for target_states in nfa.transitions.values() {
        for &to_state in target_states {
            all_nfa_states.insert(to_state);
        }
    }
    for &accepting_state in &nfa.accepting_states {
        all_nfa_states.insert(accepting_state);
    }

    let start_state = all_nfa_states.iter().max().copied().unwrap_or(0);

    let start_set: BTreeSet<u32> = {
        let mut set = BTreeSet::new();
        set.insert(start_state);
        set
    };

    state_sets_to_dfa_state.insert(start_set.clone(), next_dfa_state);
    queue.push_back(start_set);
    next_dfa_state += 1;

    while let Some(current_set) = queue.pop_front() {
        let current_dfa_state = state_sets_to_dfa_state[&current_set];

        // Check if this DFA state should be accepting
        if current_set
            .iter()
            .any(|&s| nfa.accepting_states.contains(&s))
        {
            dfa_accepting_states.insert(current_dfa_state);
        }

        // For each symbol in alphabet
        for &symbol in &alphabet {
            let mut next_set = BTreeSet::new();

            // Collect all states reachable via this symbol
            for &state in &current_set {
                if let Some(targets) = nfa.transitions.get(&(state, symbol)) {
                    next_set.extend(targets);
                }
            }

            if !next_set.is_empty() {
                let next_dfa_state = if let Some(&existing) = state_sets_to_dfa_state.get(&next_set)
                {
                    existing
                } else {
                    let new_state = next_dfa_state;
                    next_dfa_state += 1;
                    state_sets_to_dfa_state.insert(next_set.clone(), new_state);
                    queue.push_back(next_set.clone());
                    new_state
                };

                dfa_transitions.insert((current_dfa_state, symbol), next_dfa_state);
            }
        }
    }

    // Normalize to start from state 0
    normalize_dfa_states(dfa_transitions, dfa_accepting_states)
}

fn normalize_dfa_states(
    transitions: HashMap<(u32, char), u32>,
    accepting_states: HashSet<u32>,
) -> GlushkovDfa {
    if transitions.is_empty() && accepting_states.is_empty() {
        return GlushkovDfa {
            transitions,
            accepting_states,
        };
    }

    // Find all states
    let mut all_states = HashSet::new();
    for &(from, _) in transitions.keys() {
        all_states.insert(from);
    }
    for &to in transitions.values() {
        all_states.insert(to);
    }
    all_states.extend(&accepting_states);

    if all_states.is_empty() {
        return GlushkovDfa {
            transitions,
            accepting_states,
        };
    }

    // Create mapping with 0 as start state
    let start_state = *all_states.iter().min().unwrap();
    let mut state_mapping = HashMap::new();
    state_mapping.insert(start_state, 0);

    let mut next_state = 1;
    for &state in &all_states {
        if state != start_state {
            state_mapping.insert(state, next_state);
            next_state += 1;
        }
    }

    // Remap transitions
    let mut new_transitions = HashMap::new();
    for ((from, symbol), to) in transitions {
        let new_from = state_mapping[&from];
        let new_to = state_mapping[&to];
        new_transitions.insert((new_from, symbol), new_to);
    }

    // Remap accepting states
    let mut new_accepting_states = HashSet::new();
    for state in accepting_states {
        new_accepting_states.insert(state_mapping[&state]);
    }

    GlushkovDfa {
        transitions: new_transitions,
        accepting_states: new_accepting_states,
    }
}

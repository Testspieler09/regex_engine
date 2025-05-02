use core::panic;
use std::collections::{HashMap, HashSet, VecDeque};

struct NFA {
    transitions: HashMap<(u32, Option<char>), Vec<u32>>,
    accepting_state: u32, // the thompson construction always has one accepting_state
}

pub struct DFA {
    transitions: HashMap<(u32, Option<char>), u32>,
    accepting_states: HashSet<u32>,
}

fn is_valid_regex(regex: &str) -> bool {
    if regex.is_empty() {
        return false;
    }

    let mut open_paren_count = 0;
    let mut last_was_quantifier = false;

    let mut chars = regex.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '(' => {
                open_paren_count += 1;
                last_was_quantifier = false;
            }

            ')' => {
                if open_paren_count == 0 {
                    return false;
                }
                open_paren_count -= 1;
                last_was_quantifier = false;
            }

            '*' | '+' => {
                // Ensure quantifiers are not the first character and are not repeated
                if last_was_quantifier || regex.starts_with('*') || regex.starts_with('+') {
                    return false;
                }
                last_was_quantifier = true;
            }

            '|' => {
                // Ensure alternation isn't the first or last character
                if regex.starts_with('|') || chars.peek().is_none() {
                    return false;
                }
                last_was_quantifier = false;
            }

            '\\' => {
                // Handle escaped characters: ensure there's a character after the escape
                if chars.peek().is_none() {
                    return false;
                }
                chars.next(); // Skip the escaped character
                last_was_quantifier = false;
            }

            _ => {
                last_was_quantifier = false;
            }
        }
    }

    open_paren_count == 0
}

fn normalise_regex(regex: &str) -> String {
    let mut normalised = String::new();
    let mut escape_sequence = false;
    let mut prev_char = '\0';

    for curr_char in regex.chars() {
        if escape_sequence {
            // TODO: Implement further parsing features here (e.g. \w \d)
            normalised.push(curr_char);
            escape_sequence = false;
            prev_char = curr_char;
            continue;
        }

        if curr_char == '\\' {
            escape_sequence = true;
            normalised.push(curr_char);
            continue;
        }

        if curr_char == '+' {
            normalised.push(prev_char);
            normalised.push('*');
            prev_char = curr_char;
            continue;
        }
        if curr_char == '?' {
            match prev_char {
                ')' => {
                    let mut balance = 0;

                    for j in (0..normalised.len()).rev() {
                        let ch = normalised.chars().nth(j).unwrap();
                        if ch == ')' {
                            balance += 1;
                        } else if ch == '(' {
                            balance -= 1;
                            if balance == 0 {
                                normalised.insert(j, '(');
                                break;
                            }
                        }
                    }
                }
                _ => {
                    normalised.insert(normalised.len() - 1, '(');
                }
            }
            normalised.push_str("|())");
            prev_char = curr_char;
            continue;
        }
        if curr_char == '.' {
            normalised.push_str("(a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|0|1|2|3|4|5|6|7|8|9| |!|\"|#|$|%|&|'|\\(|\\)|\\*|\\+|,|-|.|/|:|;|<|=|>|?|@|[|\\\\|]|^|_|`|{|}|~)");
            prev_char = curr_char;
            continue;
        }

        normalised.push(curr_char);
        prev_char = curr_char;
    }

    normalised
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

// THOMPSON CONSTRUCTION ---
fn thompson_construction(normalised_regex: &str) -> NFA {
    fn apply_operator(nfa_stack: &mut Vec<NFA>, operator: char) {
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
            _ => panic!("Unknown operator {:?}", operator),
        }
    }

    let mut operators: Vec<char> = Vec::new();
    let mut nfa_stack: Vec<NFA> = Vec::new();
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
                let mut is_epsilon = true;
                while let Some(op) = operators.pop() {
                    if op == '(' && is_epsilon {
                        nfa_stack.push(create_basic_epsilon_nfa());
                        break;
                    } else if op == '(' {
                        break;
                    }
                    is_epsilon = false;
                    apply_operator(&mut nfa_stack, op);
                }
                concat_flag = true;
            }
            '*' => {
                let last_nfa = nfa_stack.pop().expect("Expected NFA for Kleene Star");
                nfa_stack.push(apply_kleene_star(&last_nfa));
                concat_flag = true;
            }
            '|' => {
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

    while let Some(op) = operators.pop() {
        apply_operator(&mut nfa_stack, op);
    }

    if nfa_stack.len() != 1 {
        panic!("Invalid Regex, unexpected final NFA stack size");
    }

    nfa_stack.pop().unwrap()
}

fn apply_kleene_star(last_nfa: &NFA) -> NFA {
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

    NFA {
        transitions,
        accepting_state: new_accepting,
    }
}

fn union(left: &NFA, right: &NFA) -> NFA {
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
        .entry((&left.accepting_state + 1, None))
        .or_insert_with(Vec::new)
        .push(new_accepting_state);
    transitions
        .entry((&right.accepting_state + num_states_left_nfa + 2, None))
        .or_insert_with(Vec::new)
        .push(new_accepting_state);

    NFA {
        transitions,
        accepting_state: new_accepting_state,
    }
}

fn concatenate(left: &NFA, right: &NFA) -> NFA {
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

    NFA {
        transitions,
        accepting_state: right.accepting_state + num_states_left_nfa,
    }
}

fn create_basic_nfa(letter: &char) -> NFA {
    NFA {
        transitions: HashMap::from([((0, Some(*letter)), vec![1])]),
        accepting_state: 1,
    }
}

fn create_basic_epsilon_nfa() -> NFA {
    NFA {
        transitions: HashMap::from([((0, None), vec![1])]),
        accepting_state: 1,
    }
}
// END THOMPSON CONSTRUCTION ---

// NFA to DFA functions ---
fn epsilon_closure(nfa: &NFA, states: &mut HashSet<u32>) {
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

fn move_nfa(nfa: &NFA, states: &HashSet<u32>, symbol: char) -> HashSet<u32> {
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

fn nfa_to_dfa(nfa: &NFA) -> DFA {
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

            transitions.insert((current_dfa_state_id, Some(symbol)), state_map[&sorted_vec]);
        }
    }

    DFA {
        transitions,
        accepting_states: dfa_accepting_states,
    }
}
// END NFA to DFA functions ---

fn optimise_dfa(dfa: &DFA) -> DFA {
    let mut partition: HashMap<u32, usize> = HashMap::new();
    let mut accepting_states_set: HashSet<u32> = dfa.accepting_states.clone();
    let mut non_accepting_states: HashSet<u32> = HashSet::new();
    let mut all_states: HashSet<u32> = HashSet::new();

    for (&(state, _), _) in &dfa.transitions {
        all_states.insert(state);
        if dfa.accepting_states.contains(&state) {
            accepting_states_set.insert(state);
        } else {
            non_accepting_states.insert(state);
        }
    }

    for state in dfa.accepting_states.iter() {
        all_states.insert(*state);
    }

    for state in all_states.iter() {
        if dfa.accepting_states.contains(state) {
            partition.insert(*state, 0);
        } else {
            partition.insert(*state, 1);
        }
    }

    let mut partition_list: Vec<HashSet<u32>> = Vec::new();
    partition_list.push(accepting_states_set);
    partition_list.push(non_accepting_states);

    let mut worklist: VecDeque<usize> = VecDeque::new();
    if partition_list[0].len() > 0 {
        worklist.push_back(0);
    }
    if partition_list.len() > 1 && partition_list[1].len() > 0 {
        worklist.push_back(1);
    }

    while let Some(current_partition_index) = worklist.pop_front() {
        let mut states_to_check: HashMap<Option<char>, HashSet<u32>> = HashMap::new();
        for (&(source_state, symbol), &target_state) in &dfa.transitions {
            if partition[&target_state] == current_partition_index {
                states_to_check
                    .entry(symbol)
                    .or_insert_with(HashSet::new)
                    .insert(source_state);
            }
        }

        for (_, states_to_split) in states_to_check.iter() {
            let mut partitions_to_split: HashSet<usize> = HashSet::new();

            for &state in states_to_split.iter() {
                let partition_index = partition[&state];
                if partition_list[partition_index].len() > 1 {
                    partitions_to_split.insert(partition_index);
                }
            }

            for &partition_index_to_split in partitions_to_split.iter() {
                let mut intersection: HashSet<u32> = HashSet::new();
                let mut difference: HashSet<u32> = HashSet::new();

                for &state in partition_list[partition_index_to_split].iter() {
                    if states_to_split.contains(&state) {
                        intersection.insert(state);
                    } else {
                        difference.insert(state);
                    }
                }

                if !intersection.is_empty() && !difference.is_empty() {
                    let new_partition_index = partition_list.len();

                    for &state in intersection.iter() {
                        partition.insert(state, new_partition_index);
                    }

                    partition_list.push(intersection);

                    for &state in &difference {
                        partition.insert(state, partition_index_to_split);
                    }
                    partition_list[partition_index_to_split] = difference;

                    if partition_list[new_partition_index].len()
                        < partition_list[partition_index_to_split].len()
                    {
                        worklist.push_back(new_partition_index);
                    } else {
                        worklist.push_back(partition_index_to_split);
                    }
                }
            }
        }
    }

    let mut minimal_transitions: HashMap<(u32, Option<char>), u32> = HashMap::new();
    let mut minimal_accepting_states: HashSet<u32> = HashSet::new();
    let mut new_state_map: HashMap<usize, u32> = HashMap::new();

    let mut next_state_id: u32 = 0;

    if let Some(partition_index) = partition.get(&0) {
        new_state_map.insert(*partition_index, next_state_id);
        next_state_id += 1;
    }

    for (_, &partition_index) in partition.iter() {
        if !new_state_map.contains_key(&partition_index) {
            new_state_map.insert(partition_index, next_state_id);
            next_state_id += 1;
        }
    }

    for (original_state, &partition_index) in partition.iter() {
        let new_state_id = new_state_map[&partition_index];
        if dfa.accepting_states.contains(original_state) {
            minimal_accepting_states.insert(new_state_id);
        }
    }

    for (&(source_state, symbol), &target_state) in &dfa.transitions {
        let source_partition = partition[&source_state];
        let target_partition = partition[&target_state];

        let new_source_state = new_state_map[&source_partition];
        let new_target_state = new_state_map[&target_partition];

        minimal_transitions.insert((new_source_state, symbol), new_target_state);
    }

    DFA {
        transitions: minimal_transitions,
        accepting_states: minimal_accepting_states,
    }
}

impl DFA {
    pub fn new(regex: &str) -> Self {
        if !is_valid_regex(regex) {
            panic!("{} is not a valid regular expression!", regex);
        }

        let normalised_regex = normalise_regex(&regex);
        let regex_nfa: NFA = thompson_construction(&normalised_regex);
        let regex_dfa = nfa_to_dfa(&regex_nfa);
        optimise_dfa(&regex_dfa)
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

    pub fn find_first_match<'a>(&self, text: &'a str) -> Option<&'a str> {
        let mut start_pos = 0;
        while start_pos < text.len() {
            let mut current_state = 0;
            let mut match_start = None;
            let mut match_end = None;
            let mut found_match = false;

            for (i, c) in text.chars().enumerate().skip(start_pos) {
                if let Some(&next_state) = self.transitions.get(&(current_state, Some(c))) {
                    current_state = next_state;
                    match_start = match_start.or(Some(i));

                    if self.accepting_states.contains(&current_state) {
                        found_match = true;
                        match_end = Some(i)
                    }

                    if i == text.len() - 1 && found_match {
                        break;
                    }
                } else {
                    break;
                }
            }

            if let (Some(start), Some(end)) = (match_start, match_end) {
                return Some(&text[start..=end]);
            } else {
                start_pos += 1;
            }
        }

        None
    }

    pub fn find_all_matches<'a>(&self, input: &'a str) -> Vec<&'a str> {
        let mut matches: Vec<&str> = Vec::new();

        let mut start_pos = 0;
        while start_pos < input.len() {
            let mut current_state = 0;
            let mut match_start: Option<usize> = None;
            let mut match_end: Option<usize> = None;
            let mut found_match = false;

            for (i, c) in input.chars().enumerate().skip(start_pos) {
                if let Some(&next_state) = self.transitions.get(&(current_state, Some(c))) {
                    current_state = next_state;
                    match_start = match_start.or(Some(start_pos));

                    if self.accepting_states.contains(&current_state) {
                        match_end = Some(i);
                        found_match = true;
                    }

                    if i == input.len() - 1 && found_match {
                        break;
                    }
                } else {
                    break;
                }
            }

            if let (Some(start), Some(end)) = (match_start, match_end) {
                matches.push(&input[start..=end]);
                start_pos = end;
            } else {
                start_pos += 1;
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_regex_basic_test() {
        let regex = "(a|b)*";
        assert!(is_valid_regex(regex), "Expected valid regex.");
    }

    #[test]
    fn invalid_empty_regex_test() {
        let regex = "";
        assert!(!is_valid_regex(regex), "Expected invalid regex (empty).");
    }

    #[test]
    fn invalid_unbalanced_parentheses_test() {
        let regex1 = "(a|b";
        let regex2 = "a|b)";
        assert!(
            !is_valid_regex(regex1),
            "Expected invalid regex (unbalanced parentheses)."
        );
        assert!(
            !is_valid_regex(regex2),
            "Expected invalid regex (unbalanced parentheses)."
        );
    }

    #[test]
    fn invalid_operator_placement_test() {
        let regex1 = "*a";
        let regex2 = "|a|b";
        assert!(
            !is_valid_regex(regex1),
            "Expected invalid regex (invalid quantifier placement)."
        );
        assert!(
            !is_valid_regex(regex2),
            "Expected invalid regex (invalid alternation placement)."
        );
    }

    #[test]
    fn valid_nested_parentheses_test() {
        let regex = "((a|b)*c)";
        assert!(
            is_valid_regex(regex),
            "Expected valid regex with nested parentheses."
        );
    }

    #[test]
    fn valid_escape_sequence_test() {
        let regex = "a\\*b";
        assert!(
            is_valid_regex(regex),
            "Expected valid regex with escape sequence."
        );
    }

    #[test]
    fn invalid_escape_sequence_test() {
        let regex = "a\\";
        assert!(
            !is_valid_regex(regex),
            "Expected invalid regex with unpaired escape."
        );
    }

    #[test]
    fn normalise_regex_test() {
        let cases = [
            (r"a+", r"aa*"),
            (r"a\+", r"a\+"),
            (r"a?", r"(a|())"),
            (r"a\?", r"a\?"),
            (r"(ab)?", r"((ab)|())"),
            (r".", "(a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|0|1|2|3|4|5|6|7|8|9| |!|\"|#|$|%|&|'|\\(|\\)|\\*|\\+|,|-|.|/|:|;|<|=|>|?|@|[|\\\\|]|^|_|`|{|}|~)"),
        ];

        for (input, expected) in cases {
            let result = normalise_regex(input);
            assert_eq!(
                result, expected,
                "Normalisation failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn create_dfa_test() {
        let generated_dfa = DFA::new("(a|b)*");
        let expected_transitions = HashMap::from([((0, Some('a')), 0), ((0, Some('b')), 0)]);
        let expected_accepting_states = HashSet::from([0]);

        assert_eq!(expected_transitions, generated_dfa.transitions);
        assert_eq!(expected_accepting_states, generated_dfa.accepting_states);

        let generated_dfa_2 = DFA::new("a|()");
        let expected_transitions_2 = HashMap::from([((0, Some('a')), 1)]);
        let expected_accepting_states_2 = HashSet::from([0, 1]);

        assert_eq!(expected_transitions_2, generated_dfa_2.transitions);
        assert_eq!(
            expected_accepting_states_2,
            generated_dfa_2.accepting_states
        );
    }

    #[test]
    fn prozess_regex_test() {
        let generated_dfa = DFA::new("(a|b)*");
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
        let input_nfa = NFA {
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

        let expected_options = vec![
            HashMap::from([
                ((0, Some('a')), 1),
                ((0, Some('b')), 2),
                ((1, Some('a')), 1),
                ((1, Some('b')), 2),
                ((2, Some('a')), 1),
                ((2, Some('b')), 2),
            ]),
            HashMap::from([
                ((0, Some('a')), 2),
                ((0, Some('b')), 1),
                ((1, Some('a')), 2),
                ((1, Some('b')), 1),
                ((2, Some('a')), 2),
                ((2, Some('b')), 1),
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

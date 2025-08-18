use crate::{glushkov::GlushkovDfa, thompson::ThompsonDfa};
use std::collections::{HashMap, HashSet, VecDeque};

mod glushkov;
mod thompson;

trait Dfa {
    fn new(regex: &str) -> Result<Self, String>
    where
        Self: std::marker::Sized;
    fn get_transitions(&self) -> &HashMap<(u32, char), u32>;
    fn get_accepting_states(&self) -> &HashSet<u32>;
    fn get_transitions_mut(&mut self) -> &mut HashMap<(u32, char), u32>;
    fn get_accepting_states_mut(&mut self) -> &mut HashSet<u32>;
    fn optimise_dfa(&mut self) {
        let mut partition: HashMap<u32, usize> = HashMap::new();
        let mut accepting_states_set: HashSet<u32> = self.get_accepting_states().clone();
        let mut non_accepting_states: HashSet<u32> = HashSet::new();
        let mut all_states: HashSet<u32> = HashSet::new();

        for &(state, _) in self.get_transitions().keys() {
            all_states.insert(state);
            if self.get_accepting_states().contains(&state) {
                accepting_states_set.insert(state);
            } else {
                non_accepting_states.insert(state);
            }
        }

        for state in self.get_accepting_states().iter() {
            all_states.insert(*state);
        }

        for state in all_states.iter() {
            if self.get_accepting_states().contains(state) {
                partition.insert(*state, 0);
            } else {
                partition.insert(*state, 1);
            }
        }

        let mut partition_list: Vec<HashSet<u32>> = Vec::new();
        partition_list.push(accepting_states_set);
        partition_list.push(non_accepting_states);

        let mut worklist: VecDeque<usize> = VecDeque::new();
        if !partition_list[0].is_empty() {
            worklist.push_back(0);
        }
        if partition_list.len() > 1 && !partition_list[1].is_empty() {
            worklist.push_back(1);
        }

        while let Some(current_partition_index) = worklist.pop_front() {
            let mut states_to_check: HashMap<char, HashSet<u32>> = HashMap::new();
            for (&(source_state, symbol), &target_state) in self.get_transitions() {
                if partition[&target_state] == current_partition_index {
                    states_to_check
                        .entry(symbol)
                        .or_default()
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

        // Build new transitions and accepting states
        let mut minimal_transitions: HashMap<(u32, char), u32> = HashMap::new();
        let mut minimal_accepting_states: HashSet<u32> = HashSet::new();
        let mut new_state_map: HashMap<usize, u32> = HashMap::new();

        let mut next_state_id: u32 = 0;

        if let Some(partition_index) = partition.get(&0) {
            new_state_map.insert(*partition_index, next_state_id);
            next_state_id += 1;
        }

        for (_, &partition_index) in partition.iter() {
            if let std::collections::hash_map::Entry::Vacant(e) =
                new_state_map.entry(partition_index)
            {
                e.insert(next_state_id);
                next_state_id += 1;
            }
        }

        for (original_state, &partition_index) in partition.iter() {
            let new_state_id = new_state_map[&partition_index];
            if self.get_accepting_states().contains(original_state) {
                minimal_accepting_states.insert(new_state_id);
            }
        }

        for (&(source_state, symbol), &target_state) in self.get_transitions() {
            let source_partition = partition[&source_state];
            let target_partition = partition[&target_state];

            let new_source_state = new_state_map[&source_partition];
            let new_target_state = new_state_map[&target_partition];

            minimal_transitions.insert((new_source_state, symbol), new_target_state);
        }

        // Modify the existing DFA in-place
        *self.get_transitions_mut() = minimal_transitions;
        *self.get_accepting_states_mut() = minimal_accepting_states;
    }

    /// Determines if the given input string exactly matches the regex pattern.
    ///
    /// This function processes the input as though it is surrounded by start (`^`) and
    /// end (`$`) position anchors, ensuring that the entire input must conform to the pattern.
    ///
    /// # Parameters
    ///
    /// - `input`: A string slice representing the text to be checked against the regex.
    ///
    /// # Returns
    ///
    /// Returns `true` if the entire input string matches the regex pattern exactly,
    /// considering implicit start and end anchors.
    ///
    /// e.g., for the regex pattern "(a|b)*", the function checks if the input matches
    /// the pattern from start to finish, equivalent to "^(a|b)*$".
    ///
    fn process(&self, input: &str) -> bool {
        let mut current_state = 0;
        for c in input.chars() {
            if let Some(&next_state) = self.get_transitions().get(&(current_state, c)) {
                current_state = next_state;
            } else {
                return false;
            }
        }
        self.get_accepting_states().contains(&current_state)
    }

    fn find_first_match<'a>(&self, text: &'a str) -> Option<&'a str> {
        let mut start_pos = 0;
        while start_pos < text.len() {
            let mut current_state = 0;
            let mut match_start = None;
            let mut match_end = None;

            for (i, c) in text.chars().enumerate().skip(start_pos) {
                if let Some(&next_state) = self.get_transitions().get(&(current_state, c)) {
                    current_state = next_state;
                    match_start = match_start.or(Some(i));

                    if self.get_accepting_states().contains(&current_state) {
                        match_end = Some(i)
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

    fn find_all_matches<'a>(&self, input: &'a str) -> Vec<&'a str> {
        let mut matches: Vec<&str> = Vec::new();

        let mut start_pos = 0;
        while start_pos < input.len() {
            let mut current_state = 0;
            let mut match_start: Option<usize> = None;
            let mut match_end: Option<usize> = None;

            for (i, c) in input.chars().enumerate().skip(start_pos) {
                if let Some(&next_state) = self.get_transitions().get(&(current_state, c)) {
                    current_state = next_state;
                    match_start = match_start.or(Some(start_pos));

                    if self.get_accepting_states().contains(&current_state) {
                        match_end = Some(i);
                    }
                } else {
                    break;
                }
            }

            if let (Some(start), Some(end)) = (match_start, match_end) {
                matches.push(&input[start..=end]);
                start_pos = end + 1;
            } else {
                start_pos += 1;
            }
        }

        matches
    }
}

pub enum ConstructionType {
    Thompson,
    Glushkov,
}

enum DfaType {
    Thompson(ThompsonDfa),
    Glushkov(GlushkovDfa),
}

pub struct Regex {
    dfa: DfaType,
}

impl Regex {
    pub fn new(pattern: &str, construction: ConstructionType) -> Result<Self, String> {
        let dfa_type = match construction {
            ConstructionType::Thompson => DfaType::Thompson(ThompsonDfa::new(pattern)?),
            ConstructionType::Glushkov => DfaType::Glushkov(GlushkovDfa::new(pattern)?),
        };
        Ok(Regex { dfa: dfa_type })
    }

    /// Determines if the provided `text` is an exact match for the regex pattern.
    ///
    /// This method interprets the regex pattern as though it is bracketed by start (`^`)
    /// and end (`$`) anchors, requiring the entire `text` to conform to the pattern.
    ///
    /// # Parameters
    ///
    /// - `text`: A string slice that represents the text to be verified against the regex.
    ///
    /// # Returns
    ///
    /// Returns `true` if the `text` completely matches the regex pattern encompassed by implicit
    /// anchors, otherwise returns `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use regex_engine::{Regex, ConstructionType};
    ///
    /// let regex = Regex::new("(a|b)*", ConstructionType::Thompson);
    /// assert!(regex.is_match("abba"));
    /// assert!(!regex.is_match("abc"));
    /// ```
    pub fn is_match(&self, text: &str) -> bool {
        match &self.dfa {
            DfaType::Thompson(dfa) => dfa.process(text),
            DfaType::Glushkov(dfa) => dfa.process(text),
        }
    }

    /// Searches for the first occurrence of a sequence in `text` that matches the regex pattern.
    ///
    /// This method locates and returns the first substring of `text` that matches the regex,
    /// if such a substring exists.
    ///
    /// # Parameters
    ///
    /// - `text`: A string slice in which to search for the regex pattern.
    ///
    /// # Returns
    ///
    /// Returns an `Option<&str>` which contains the first matching substring if a match is found,
    /// or `None` if no match occurs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use regex_engine::{Regex, ConstructionType};
    ///
    /// let regex = Regex::new("ab+", ConstructionType::Thompson);
    /// if let Some(matched) = regex.find("aabbcc") {
    ///     println!("Found: {}", matched);
    /// }
    /// // Output: Found: abb
    /// ```
    pub fn find<'a>(&self, text: &'a str) -> Option<&'a str> {
        match &self.dfa {
            DfaType::Thompson(dfa) => dfa.find_first_match(text),
            DfaType::Glushkov(dfa) => dfa.find_first_match(text),
        }
    }

    pub fn findall<'a>(&self, text: &'a str) -> Vec<&'a str> {
        match &self.dfa {
            DfaType::Thompson(dfa) => dfa.find_all_matches(text),
            DfaType::Glushkov(dfa) => dfa.find_all_matches(text),
        }
    }
}

pub fn is_valid_regex(regex: &str) -> bool {
    if regex.is_empty() {
        return false;
    }

    let mut open_paren_count = 0;
    let mut last_was_quantifier = true;

    let mut chars = regex.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '(' => {
                open_paren_count += 1;
                last_was_quantifier = true;
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
                if last_was_quantifier {
                    return false;
                }
                last_was_quantifier = true;
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

pub fn normalise_regex(regex: &str) -> String {
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
            match prev_char {
                ')' => {
                    let mut balance = 0;
                    let mut group_start = 0;

                    for j in (0..normalised.len()).rev() {
                        let ch = normalised.chars().nth(j).unwrap();
                        if ch == ')' {
                            balance += 1;
                        } else if ch == '(' {
                            balance -= 1;
                            if balance == 0 {
                                group_start = j;
                                break;
                            }
                        }
                    }

                    let group = String::from(&normalised[group_start..normalised.len()]);
                    normalised.push_str(&group);
                }
                _ => {
                    normalised.push(prev_char);
                }
            }
            normalised.push('*');
            prev_char = '*';
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
            normalised.push_str("|)");
            prev_char = ')';
            continue;
        }
        if curr_char == '.' {
            normalised.push_str("(a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|0|1|2|3|4|5|6|7|8|9| |!|\"|#|$|%|&|'|\\(|\\)|\\*|\\+|,|-|.|/|:|;|<|=|>|?|@|[|\\\\|]|^|_|`|{|}|~)");
            prev_char = ')';
            continue;
        }
        normalised.push(curr_char);
        prev_char = curr_char;
    }
    normalised
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
        let regex2 = "(+abc|x)";
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
            (r"a?", r"(a|)"),
            (r"a\?", r"a\?"),
            (r"(ab)?", r"((ab)|)"),
            (
                r".",
                "(a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|0|1|2|3|4|5|6|7|8|9| |!|\"|#|$|%|&|'|\\(|\\)|\\*|\\+|,|-|.|/|:|;|<|=|>|?|@|[|\\\\|]|^|_|`|{|}|~)",
            ),
        ];

        for (input, expected) in cases {
            let result = normalise_regex(input);
            assert_eq!(result, expected, "Normalisation failed for input '{input}'");
        }
    }

    #[test]
    fn is_match_test() {
        let regex_object = Regex::new("a(a|b)*", ConstructionType::Thompson).expect("Valid regex");

        let success_strings = vec!["abababaaaababa", "a"];
        for string in success_strings {
            assert!(regex_object.is_match(string));
        }

        let failing_strings = vec!["abc", "x"];
        for string in failing_strings {
            assert!(!regex_object.is_match(string));
        }
    }

    #[test]
    fn find_test() {
        let regex_object = Regex::new("abc", ConstructionType::Thompson).expect("Valid regex");
        let test_cases = vec![
            ("abcd", Some("abc")),
            ("xyzabc", Some("abc")),
            ("abc", Some("abc")),
            ("ac", None),
            ("def", None),
            ("aabc", Some("abc")),
        ];

        for (text, expected) in test_cases {
            let result = regex_object.find(text);
            assert_eq!(result, expected, "Failed for input: {text}");
        }
    }

    #[test]
    fn find_all_test() {
        let regex_object = Regex::new("abc*", ConstructionType::Thompson).expect("Valid regex");
        let test_cases = vec![
            ("abcd", vec!["abc"]),
            ("ac", vec![]),
            ("abcab", vec!["abc", "ab"]),
        ];

        for (text, expected) in test_cases {
            let result = regex_object.findall(text);
            assert_eq!(result, expected, "Failed for input: {text}");
        }
    }
}

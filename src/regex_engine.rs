use crate::glushkov::DFA as GlushkovDFA;
use crate::thompson::DFA as ThompsonDFA;

pub enum ConstructionType {
    Thompson,
    Glushkov,
}

enum DFAType {
    Thompson(ThompsonDFA),
    Glushkov(GlushkovDFA),
}

pub struct Regex {
    dfa: DFAType,
}

impl Regex {
    pub fn new(pattern: &str, construction: ConstructionType) -> Self {
        let dfa_type = match construction {
            ConstructionType::Thompson => DFAType::Thompson(ThompsonDFA::new(pattern)),
            ConstructionType::Glushkov => DFAType::Glushkov(GlushkovDFA::new(pattern)),
        };
        Regex { dfa: dfa_type }
    }

    pub fn is_match(&self, text: &str) -> bool {
        match &self.dfa {
            DFAType::Thompson(dfa) => dfa.process(text),
            DFAType::Glushkov(dfa) => dfa.process(text),
        }
    }

    pub fn find<'a>(&self, text: &'a str) -> Option<&'a str> {
        match &self.dfa {
            DFAType::Thompson(dfa) => dfa.find_first_match(text),
            DFAType::Glushkov(dfa) => dfa.find_first_match(text),
        }
    }

    pub fn findall<'a>(&self, text: &'a str) -> Vec<&'a str> {
        match &self.dfa {
            DFAType::Thompson(dfa) => dfa.find_all_matches(text),
            DFAType::Glushkov(dfa) => dfa.find_all_matches(text),
        }
    }
}

pub fn is_valid_regex(regex: &str) -> bool {
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
    fn is_match_test() {
        let regex_object = Regex::new("(a|b)*", ConstructionType::Thompson);

        let success_strings = vec!["abababaaaababa", ""];
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
        let regex_object = Regex::new("abc", ConstructionType::Thompson);
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
            assert_eq!(result, expected, "Failed for input: {}", text);
        }
    }

    #[test]
    fn find_all_test() {
        let regex_object = Regex::new("abc*", ConstructionType::Thompson);
        let test_cases = vec![
            ("abcd", vec!["abc"]),
            ("ac", vec![]),
            ("abcab", vec!["abc", "ab"]),
        ];

        for (text, expected) in test_cases {
            let result = regex_object.findall(text);
            assert_eq!(result, expected, "Failed for input: {}", text);
        }
    }
}

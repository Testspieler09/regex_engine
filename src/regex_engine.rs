use crate::dfa::DFA;

pub struct Regex {
    dfa: DFA,
}

impl Regex {
    pub fn new(pattern: &str) -> Self {
        Regex {
            dfa: DFA::new(pattern),
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.dfa.process(text)
    }

    pub fn find<'a>(&self, text: &'a str) -> Option<&'a str> {
        self.dfa.find_first_match(text)
    }

    pub fn findall<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.dfa.find_all_matches(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_match_test() {
        let regex_object = Regex::new("(a|b)*");

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
        let regex_object = Regex::new("abc");
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
        let regex_object = Regex::new("abc*");
        let test_cases = vec![
            ("abcd", vec!["ab"]),
            ("ac", vec![]),
            ("abcab", vec!["ab", "ab"]),
        ];

        for (text, expected) in test_cases {
            let result = regex_object.findall(text);
            assert_eq!(result, expected, "Failed for input: {}", text);
        }
    }
}

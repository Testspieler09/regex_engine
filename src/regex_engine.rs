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

    pub fn find(&self, text: &str) -> Option<&str> {
        // TODO: Find first match in the text
        todo!()
    }

    pub fn findall(&self, text: &str) -> Vec<&str> {
        // TODO: Find all matches in the text
        todo!()
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
}

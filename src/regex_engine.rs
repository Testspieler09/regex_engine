pub struct Regex {
    pattern: Vec<char>,
}

impl Regex {
    pub fn new(pattern: &str) -> Self {
        Regex { pattern: pattern.chars().collect() }
    }

    pub fn is_match(&self, text: &str) -> bool {
        // TODO: Check if the text matches the regex
        // This should run the finite automaton.
        true
    }

    pub fn find(&self, text: &str) -> Option<&str> {
        // TODO: Find first match in the text
        None
    }

    pub fn findall(&self, text: &str) -> Vec<&str> {
        // TODO: Find all matches in the text
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find() {
        assert!(true);
    }
}

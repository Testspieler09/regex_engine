pub struct Regex {
    pattern: Vec<char>,
}

impl Regex {
    pub fn new(pattern: &str) -> Self {
        Regex {
            pattern: pattern.chars().collect(),
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        // TODO: Check if the text matches the regex
        // This should run the finite automaton.
        todo!();
    }

    pub fn find(&self, text: &str) -> Option<&str> {
        // TODO: Find first match in the text
        todo!();
    }

    pub fn findall(&self, text: &str) -> Vec<&str> {
        // TODO: Find all matches in the text
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_test() {
        todo!();
    }
}

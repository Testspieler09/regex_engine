use std::collections::{HashMap, HashSet};

mod glushkov;
pub mod regex_engine;
mod thompson;

trait Dfa {
    fn new(regex: &str) -> Self;
    fn get_transitions(&self) -> &HashMap<(u32, char), u32>;
    fn get_accepting_states(&self) -> &HashSet<u32>;
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
            let mut found_match = false;

            for (i, c) in text.chars().enumerate().skip(start_pos) {
                if let Some(&next_state) = self.get_transitions().get(&(current_state, c)) {
                    current_state = next_state;
                    match_start = match_start.or(Some(i));

                    if self.get_accepting_states().contains(&current_state) {
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

    fn find_all_matches<'a>(&self, input: &'a str) -> Vec<&'a str> {
        let mut matches: Vec<&str> = Vec::new();

        let mut start_pos = 0;
        while start_pos < input.len() {
            let mut current_state = 0;
            let mut match_start: Option<usize> = None;
            let mut match_end: Option<usize> = None;
            let mut found_match = false;

            for (i, c) in input.chars().enumerate().skip(start_pos) {
                if let Some(&next_state) = self.get_transitions().get(&(current_state, c)) {
                    current_state = next_state;
                    match_start = match_start.or(Some(start_pos));

                    if self.get_accepting_states().contains(&current_state) {
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

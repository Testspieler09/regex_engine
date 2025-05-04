use regex_engine::regex_engine::{ConstructionType, Regex};

#[test]
fn test_escape_sequence_plus() {
    let pattern = r"a*b\+";
    let text = "aaab+b"; // should fail on match
    let text_success = "aaab+";

    let engine = Regex::new(pattern, ConstructionType::Thompson);

    let expected_match = text_success;

    assert!(!engine.is_match(text));
    assert!(engine.is_match(text_success));
    assert_eq!(engine.find(text), Some(expected_match));
    assert_eq!(engine.findall(text), vec![expected_match]);
}

#[test]
fn test_escape_sequence_slash() {
    let pattern = r"a*b\\";
    let text = "aaab\\b"; // should fail on match
    let text_success = "aaab\\";

    let engine = Regex::new(pattern, ConstructionType::Thompson);

    let expected_match = text_success;

    assert!(!engine.is_match(text));
    assert!(engine.is_match(text_success));
    assert_eq!(engine.find(text), Some(expected_match));
    assert_eq!(engine.findall(text), vec![expected_match]);
}

#[test]
fn test_dot_wildcard() {
    let pattern = r"a.*";
    let text = "cabbc"; // should fail on match
    let text_success = "abbc";

    let engine = Regex::new(pattern, ConstructionType::Thompson);

    let expected_match = text_success;

    assert!(!engine.is_match(text));
    assert!(engine.is_match(text_success));
    assert_eq!(engine.find(text), Some(expected_match));
    assert_eq!(engine.findall(text), vec![expected_match]);
}

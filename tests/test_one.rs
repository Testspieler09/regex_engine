use regex_engine::regex_engine::Regex;

#[test]
fn test_escape_sequence() {
    let pattern = r"a*b\+";
    let text = "aaab+b"; // should fail on match
    let text_success = "aaab+";

    let engine = Regex::new(pattern);

    let expected_match = "aaab+";

    assert!(!engine.is_match(text));
    assert!(engine.is_match(text_success));
    assert_eq!(engine.find(text), Some(expected_match));
    assert_eq!(engine.findall(text), vec![expected_match]);
}

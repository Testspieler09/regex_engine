include!("../benches/bench_cases.rs");
use regex::Regex;

#[test]
fn test_all_bench_cases() {
    let cases = get_bench_cases();

    for case in &cases {
        let match_regex = Regex::new(format!("^{}$", case.regex).as_str())
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));
        let regex = Regex::new(case.regex)
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        assert_eq!(match_regex.is_match(&case.input), case.expected_is_match);
        assert_eq!(
            regex.find(&case.input).map(|s| s.as_str()),
            case.expected_first_match.as_deref()
        );
        assert_eq!(
            regex
                .find_iter(&case.input)
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            case.expected_all_matches
        );
    }
}

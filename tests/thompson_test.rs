include!("../benches/bench_cases.rs");
use regex_engine::{ConstructionType, Regex};

#[test]
fn test_all_bench_cases() {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Thompson);

        assert_eq!(regex.is_match(&case.input), case.expected_is_match);
        assert_eq!(
            regex.find(&case.input),
            case.expected_first_match.as_deref()
        );
        assert_eq!(regex.findall(&case.input), case.expected_all_matches);
    }
}

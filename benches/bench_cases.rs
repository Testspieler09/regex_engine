use std::hint::black_box;

#[allow(dead_code)]
struct BenchCase<'a> {
    pub regex: &'a str,
    pub input: String,
    pub expected_is_match: bool,
    pub expected_first_match: Option<String>,
    pub expected_all_matches: Vec<String>,
}

// This function is used in the `benchmark` files
#[allow(dead_code)]
fn get_bench_cases() -> Vec<BenchCase<'static>> {
    black_box(vec![
        BenchCase {
            regex: r"a.b",
            input: "abcd abef abgh ijk".to_string(),
            expected_is_match: false,
            expected_first_match: None,
            expected_all_matches: vec![],
        },
        BenchCase {
            regex: r"a*b",
            input: "aaaaaaaaab".to_string(),
            expected_is_match: true,
            expected_first_match: Some("aaaaaaaaab".to_string()),
            expected_all_matches: vec!["aaaaaaaaab".to_string()],
        },
        BenchCase {
            regex: r"a+b",
            input: "aabab".to_string(),
            expected_is_match: false,
            expected_first_match: Some("aab".to_string()),
            expected_all_matches: vec!["aab".to_string(), "ab".to_string()],
        },
        BenchCase {
            regex: r"a?b",
            input: "b aaab ab".to_string(),
            expected_is_match: false,
            expected_first_match: Some("b".to_string()),
            expected_all_matches: vec!["b".to_string(), "ab".to_string(), "ab".to_string()],
        },
        BenchCase {
            regex: r"a|b",
            input: "xxaxybxx".to_string(),
            expected_is_match: false,
            expected_first_match: Some("a".to_string()),
            expected_all_matches: vec!["a".to_string(), "b".to_string()],
        },
        BenchCase {
            regex: r"(a|b)c",
            input: "abc ac bc bbcc".to_string(),
            expected_is_match: false,
            expected_first_match: Some("bc".to_string()),
            expected_all_matches: vec![
                "bc".to_string(),
                "ac".to_string(),
                "bc".to_string(),
                "bc".to_string(),
            ],
        },
        BenchCase {
            regex: r"\.",
            input: "Find . within this !?. sentence.".to_string(),
            expected_is_match: false,
            expected_first_match: Some(".".to_string()),
            expected_all_matches: vec![".".to_string(), ".".to_string(), ".".to_string()],
        },
        BenchCase {
            regex: r"(hel+o|wor?ld)",
            input: "hello helolllo world worlld helloworld".to_string(),
            expected_is_match: false,
            expected_first_match: Some("hello".to_string()),
            expected_all_matches: vec![
                "hello".to_string(),
                "helo".to_string(),
                "world".to_string(),
                "hello".to_string(),
                "world".to_string(),
            ],
        },
        BenchCase {
            regex: r"ab*c+",
            input: "abbc abbbbbbbcc bccaaabbabc".to_string(),
            expected_is_match: false,
            expected_first_match: Some("abbc".to_string()),
            expected_all_matches: vec![
                "abbc".to_string(),
                "abbbbbbbcc".to_string(),
                "abc".to_string(),
            ],
        },
        BenchCase {
            regex: r"(a(bc|de)+)",
            input: "abc abcbc abcdedef".to_string(),
            expected_is_match: false,
            expected_first_match: Some("abc".to_string()),
            expected_all_matches: vec![
                "abc".to_string(),
                "abcbc".to_string(),
                "abcdede".to_string(),
            ],
        },
        BenchCase {
            regex: r"(a|b)*c",
            input: format!("{}{}", "a".repeat(1000), "bc"),
            expected_is_match: true,
            expected_first_match: Some(format!("{}{}", "a".repeat(1000), "bc")),
            expected_all_matches: vec![format!("{}{}", "a".repeat(1000), "bc")],
        },
    ])
}

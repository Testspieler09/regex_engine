#[allow(dead_code)]
struct BenchCase<'a> {
    pub regex: &'a str,
    pub input: String,
}

// This function is used in the `benchmark` files
#[allow(dead_code)]
fn get_bench_cases() -> Vec<BenchCase<'static>> {
    vec![
        BenchCase {
            regex: r"a.b",
            input: "abcd abef abgh ijk".to_string(),
        },
        BenchCase {
            regex: r"a*b",
            input: "aaaaaaaaab".to_string(),
        },
        BenchCase {
            regex: r"a+b",
            input: "aabab".to_string(),
        },
        BenchCase {
            regex: r"a?b",
            input: "b aaab ab".to_string(),
        },
        BenchCase {
            regex: r"a|b",
            input: "xxaxybxx".to_string(),
        },
        // Group and escape sequences
        BenchCase {
            regex: r"(a|b)c",
            input: "abc ac bc bbcc".to_string(),
        },
        BenchCase {
            regex: r"\.",
            input: "Find . within this !?. sentence.".to_string(),
        },
        // Larger and more complex patterns
        BenchCase {
            regex: r"(hel+o|wor?ld)",
            input: "hello helolllo world worlld helloworld".to_string(),
        },
        BenchCase {
            regex: r"ab*c+",
            input: "abbc abbbbbbbcc bccaaabbabc".to_string(),
        },
        BenchCase {
            regex: r"(a(bc|de)+)",
            input: "abc abcbc abcdedef".to_string(),
        },
        // Realistic text patterns and larger inputs
        BenchCase {
            regex: r"\b[0-9]{2}\b",
            input: "There are 99 bottles of soda and 45 cans of juice".to_string(),
        },
        BenchCase {
            regex: r"\b\w{5,}\b",
            input: "Rust is great for systems programming but can be challenging".to_string(),
        },
        BenchCase {
            regex: r"(https?|ftp)://[^\s/$.?#].[^\s]*",
            input: "Check https://example.com out and ftp://fileserver.net as well".to_string(),
        },
        // Pathological case to test limits
        BenchCase {
            regex: r"(a|b)*c",
            input: format!("{}{}", "a".repeat(1000), "bc"),
        },
        BenchCase {
            regex: r"x{3}(y|z)",
            input: "xxxxyxxxzxxxy".to_string(),
        },
    ]
}

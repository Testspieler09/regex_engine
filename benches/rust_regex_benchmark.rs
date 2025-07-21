include!("bench_cases.rs");
use criterion::{Criterion, criterion_group, criterion_main};
use regex::Regex;

fn benchmark_rust_regex_process(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in cases {
        let regex = Regex::new(case.regex)
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        c.bench_function(&format!("Rust process match: {}", case.regex), |b| {
            b.iter(|| {
                regex.is_match(&case.input);
            })
        });
    }
}

fn benchmark_rust_regex_find_first(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in cases {
        let regex = Regex::new(case.regex)
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        c.bench_function(&format!("Rust find first match: {}", case.regex), |b| {
            b.iter(|| {
                regex.find(&case.input).map(|m| m.as_str());
            })
        });
    }
}

fn benchmark_rust_regex_find_all(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in cases {
        let regex = Regex::new(case.regex)
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        c.bench_function(&format!("Rust find all matches: {}", case.regex), |b| {
            b.iter(|| {
                regex.find_iter(&case.input);
            })
        });
    }
}

criterion_group!(
    benches,
    benchmark_rust_regex_process,
    benchmark_rust_regex_find_first,
    benchmark_rust_regex_find_all
);
criterion_main!(benches);

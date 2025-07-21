include!("bench_cases.rs");
use criterion::{Criterion, criterion_group, criterion_main};
use regex_engine::{ConstructionType, Regex};

fn benchmark_glushkov_regex_process(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Glushkov);

        c.bench_function(
            &format!("Glushkov is_match - pattern: {}", case.regex),
            |b| {
                b.iter(|| {
                    regex.is_match(&case.input);
                })
            },
        );
    }
}

fn benchmark_glushkov_regex_find_first(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Glushkov);

        c.bench_function(
            &format!("Glushkov find match - pattern: {}", case.regex),
            |b| {
                b.iter(|| {
                    regex.find(&case.input);
                })
            },
        );
    }
}

fn benchmark_glushkov_regex_find_all(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Glushkov);

        c.bench_function(
            &format!("Glushkov findall matches - pattern: {}", case.regex),
            |b| {
                b.iter(|| {
                    regex.findall(&case.input);
                })
            },
        );
    }
}

criterion_group!(
    benches,
    benchmark_glushkov_regex_process,
    benchmark_glushkov_regex_find_first,
    benchmark_glushkov_regex_find_all
);
criterion_main!(benches);

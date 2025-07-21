include!("bench_cases.rs");
use criterion::{Criterion, criterion_group, criterion_main};
use regex_engine::{ConstructionType, Regex};

fn benchmark_thompson_regex_process(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Thompson);

        c.bench_function(
            &format!("Thompson is_match - pattern: {}", case.regex),
            |b| {
                b.iter(|| {
                    regex.is_match(&case.input);
                })
            },
        );
    }
}

fn benchmark_thompson_regex_find_first(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Thompson);

        c.bench_function(
            &format!("Thompson find match - pattern: {}", case.regex),
            |b| {
                b.iter(|| {
                    regex.find(&case.input);
                })
            },
        );
    }
}

fn benchmark_thompson_regex_find_all(c: &mut Criterion) {
    let cases = get_bench_cases();

    for case in &cases {
        let regex = Regex::new(case.regex, ConstructionType::Thompson);

        c.bench_function(
            &format!("Thompson findall matches - pattern: {}", case.regex),
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
    benchmark_thompson_regex_process,
    benchmark_thompson_regex_find_first,
    benchmark_thompson_regex_find_all
);
criterion_main!(benches);

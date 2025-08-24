include!("bench_cases.rs");
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use regex as rust_regex;
use regex_engine::{ConstructionType, Regex};

fn benchmark_regex_compile_time(c: &mut Criterion) {
    let cases = get_bench_cases();
    let mut group = c.benchmark_group("Regex Compile Time");

    for case in cases {
        group.bench_with_input(
            BenchmarkId::new("Thompson", case.regex),
            &case.regex,
            |b, regex| {
                b.iter(|| {
                    let _ = Regex::new(regex, ConstructionType::Thompson);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Glushkov", case.regex),
            &case.regex,
            |b, regex| {
                b.iter(|| {
                    let _ = Regex::new(regex, ConstructionType::Glushkov);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Rust", case.regex),
            &case.regex,
            |b, regex| {
                b.iter(|| {
                    rust_regex::Regex::new(regex)
                        .unwrap_or_else(|_| panic!("Failed to create pattern: {regex}"));
                })
            },
        );
    }
    group.finish();
}

fn benchmark_regex_is_match(c: &mut Criterion) {
    let cases = get_bench_cases();
    let mut group = c.benchmark_group("Regex Is Match");

    for case in &cases {
        let thompson_regex =
            Regex::new(case.regex, ConstructionType::Thompson).expect("Valid regex");
        let glushkov_regex =
            Regex::new(case.regex, ConstructionType::Glushkov).expect("Valid regex");
        let rust_regex = rust_regex::Regex::new(&format!("^{}$", case.regex))
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        group.bench_with_input(
            BenchmarkId::new("Thompson", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    thompson_regex.is_match(input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Glushkov", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    glushkov_regex.is_match(input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Rust", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    rust_regex.is_match(input);
                })
            },
        );
    }
    group.finish();
}

fn benchmark_regex_find_first(c: &mut Criterion) {
    let cases = get_bench_cases();
    let mut group = c.benchmark_group("Regex Find First");

    for case in &cases {
        let thompson_regex =
            Regex::new(case.regex, ConstructionType::Thompson).expect("Valid regex");
        let glushkov_regex =
            Regex::new(case.regex, ConstructionType::Glushkov).expect("Valid regex");
        let rust_regex = rust_regex::Regex::new(case.regex)
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        group.bench_with_input(
            BenchmarkId::new("Thompson", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    thompson_regex.find(input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Glushkov", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    glushkov_regex.find(input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Rust", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    rust_regex.find(input).map(|m| m.as_str());
                })
            },
        );
    }
    group.finish();
}

fn benchmark_regex_find_all(c: &mut Criterion) {
    let cases = get_bench_cases();
    let mut group = c.benchmark_group("Regex Find All");

    for case in &cases {
        let thompson_regex =
            Regex::new(case.regex, ConstructionType::Thompson).expect("Valid regex");
        let glushkov_regex =
            Regex::new(case.regex, ConstructionType::Glushkov).expect("Valid regex");
        let rust_regex = rust_regex::Regex::new(case.regex)
            .unwrap_or_else(|_| panic!("Failed to create pattern: {}", case.regex));

        group.bench_with_input(
            BenchmarkId::new("Thompson", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    thompson_regex.findall(input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Glushkov", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    glushkov_regex.findall(input);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Rust", case.regex),
            &case.input,
            |b, input| {
                b.iter(|| {
                    rust_regex.find_iter(input);
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_regex_compile_time,
    benchmark_regex_is_match,
    benchmark_regex_find_first,
    benchmark_regex_find_all
);
criterion_main!(benches);

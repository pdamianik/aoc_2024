use criterion::{black_box, Criterion, criterion_group, criterion_main};

use aoc_2024::days::day3 as day;
use day::DAY;

const OFFICIAL_INPUT: &str = include_str!("../input/day3.in");

pub fn bench_parsing_official(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} parsing official input"), |b| {
        b.iter(|| {
            let _input: day::Input = black_box(OFFICIAL_INPUT.parse().unwrap());
        });
    });
}

pub fn bench_part1_official(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 1 official input"), |b| {
        let input = black_box(OFFICIAL_INPUT.parse().unwrap());
        b.iter(|| day::process_part1(black_box(&input)));
    });
}

pub fn bench_part2_official(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY}, part 2 official input"), |b| {
        let input = black_box(OFFICIAL_INPUT.parse().unwrap());
        b.iter(|| day::process_part1(black_box(&input)));
    });
}

pub fn bench_part1_official_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 1 official input with parsing"), |b| {
        b.iter(|| {
            let input = black_box(OFFICIAL_INPUT.parse().unwrap());
            day::process_part1(black_box(&input))
        });
    });
}

pub fn bench_part2_official_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 2 official input with parsing"), |b| {
        b.iter(|| {
            let input = black_box(OFFICIAL_INPUT.parse().unwrap());
            day::process_part1(black_box(&input))
        });
    });
}

criterion_group!(name = benches;
    config = Criterion::default().with_plots();
    targets =
    bench_parsing_official, bench_part1_official, bench_part2_official, bench_part1_official_with_parsing, bench_part2_official_with_parsing,
);
criterion_main!(benches);

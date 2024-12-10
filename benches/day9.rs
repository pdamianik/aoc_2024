use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aoc_2024::days::day9 as day;
use day::DAY;

const OFFICIAL_INPUT: &str = include_str!("../input/day9.in");

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

const EVIL1: &str = include_str!("../test/input/day9_evil1.in");

pub fn bench_parsing_evil1(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} parsing evil input 1"), |b| {
        b.iter(|| {
            let _input: day::Input = black_box(EVIL1.parse().unwrap());
        });
    });
}

pub fn bench_part1_evil1(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 1 evil input 1"), |b| {
        let input = EVIL1.parse().unwrap();
        b.iter(|| day::process_part1(black_box(&input)));
    });
}

pub fn bench_part2_evil1(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 2 evil input 1"), |b| {
        let input = EVIL1.parse().unwrap();
        b.iter(|| day::process_part2(black_box(&input)));
    });
}

pub fn bench_part1_evil1_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 1 evil input 1 with parsing"), |b| {
        b.iter(|| {
            let input = EVIL1.parse().unwrap();
            day::process_part1(black_box(&input))
        });
    });
}

pub fn bench_part2_evil1_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 2 evil input 1 with parsing"), |b| {
        b.iter(|| {
            let input = EVIL1.parse().unwrap();
            day::process_part2(black_box(&input))
        });
    });
}

const EVIL2: &str = include_str!("../test/input/day9_evil2.in");

pub fn bench_parsing_evil2(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} parsing evil input 2"), |b| {
        b.iter(|| {
            let _input: day::Input = black_box(EVIL2.parse().unwrap());
        });
    });
}

pub fn bench_part1_evil2(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 1 evil input 2"), |b|{
        let input = EVIL2.parse().unwrap();
        b.iter(|| day::process_part1(black_box(&input)));
    });
}

pub fn bench_part2_evil2(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 2 evil input 2"), |b| {
        let input = EVIL2.parse().unwrap();
        b.iter(|| day::process_part2(black_box(&input)));
    });
}

pub fn bench_part1_evil2_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 1 evil input 2 with parsing"), |b|{
        b.iter(|| {
            let input = EVIL2.parse().unwrap();
            day::process_part1(black_box(&input))
        });
    });
}

pub fn bench_part2_evil2_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function(&format!("{DAY} part 2 evil input 2 with parsing"), |b| {
        b.iter(|| {
            let input = EVIL2.parse().unwrap();
            day::process_part2(black_box(&input))
        });
    });
}

criterion_group!(name = benches;
    config = Criterion::default().with_plots();
    targets =
    bench_parsing_official, bench_part1_official, bench_part2_official, bench_part1_official_with_parsing, bench_part2_official_with_parsing,
    bench_parsing_evil1, bench_part1_evil1, bench_part2_evil1, bench_part1_evil1_with_parsing, bench_part2_evil1_with_parsing,
    bench_parsing_evil2, bench_part1_evil2, bench_part1_evil2, bench_part1_evil2_with_parsing, bench_part2_evil2_with_parsing,
);
criterion_main!(benches);

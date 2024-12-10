use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aoc_2024::days::day9;

pub fn bench_parsing_official(criterion: &mut Criterion) {
    criterion.bench_function("parsing official input", |b| {
        b.iter(|| {
            let _input: day9::Input = black_box(include_str!("../input/day9.in").parse().unwrap());
        });
    });
}

pub fn bench_parsing_evil1(criterion: &mut Criterion) {
    criterion.bench_function("parsing evil input 1", |b| {
        b.iter(|| {
            let _input: day9::Input = black_box(include_str!("../test/input/day9_evil1.in").parse().unwrap());
        });
    });
}

pub fn bench_parsing_evil2(criterion: &mut Criterion) {
    criterion.bench_function("parsing evil input 2", |b| {
        b.iter(|| {
            let _input: day9::Input = black_box(include_str!("../test/input/day9_evil2.in").parse().unwrap());
        });
    });
}

pub fn bench_part1_official(criterion: &mut Criterion) {
    criterion.bench_function("part 1 official input", |b| {
        let input = black_box(include_str!("../input/day9.in").parse().unwrap());
        b.iter(|| day9::process_part1(black_box(&input)));
    });
}

pub fn bench_part1_evil1(criterion: &mut Criterion) {
    criterion.bench_function("part 1 evil input 1", |b| {
        let input = include_str!("../test/input/day9_evil1.in").parse().unwrap();
        b.iter(|| day9::process_part1(black_box(&input)));
    });
}

pub fn bench_part1_evil2(criterion: &mut Criterion) {
    criterion.bench_function("part 1 evil input 2", |b|{
        let input = include_str!("../test/input/day9_evil2.in").parse().unwrap();
        b.iter(|| day9::process_part1(black_box(&input)));
    });
}

pub fn bench_part2_official(criterion: &mut Criterion) {
    criterion.bench_function("part 2 official input", |b| {
        let input = black_box(include_str!("../input/day9.in").parse().unwrap());
        b.iter(|| day9::process_part1(black_box(&input)));
    });
}

pub fn bench_part2_evil1(criterion: &mut Criterion) {
    criterion.bench_function("part 2 evil input 1", |b| {
        let input = include_str!("../test/input/day9_evil1.in").parse().unwrap();
        b.iter(|| day9::process_part2(black_box(&input)));
    });
}

pub fn bench_part2_evil2(criterion: &mut Criterion) {
    criterion.bench_function("part 2 evil input 2", |b| {
        let input = include_str!("../test/input/day9_evil2.in").parse().unwrap();
        b.iter(|| day9::process_part2(black_box(&input)));
    });
}

pub fn bench_part1_official_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function("part 1 official input with parsing", |b| {
        b.iter(|| {
            let input = black_box(include_str!("../input/day9.in").parse().unwrap());
            day9::process_part1(black_box(&input))
        });
    });
}

pub fn bench_part1_evil1_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function("part 1 evil input 1 with parsing", |b| {
        b.iter(|| {
            let input = include_str!("../test/input/day9_evil1.in").parse().unwrap();
            day9::process_part1(black_box(&input))
        });
    });
}

pub fn bench_part1_evil2_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function("part 1 evil input 2 with parsing", |b|{
        b.iter(|| {
            let input = include_str!("../test/input/day9_evil2.in").parse().unwrap();
            day9::process_part1(black_box(&input))
        });
    });
}
pub fn bench_part2_official_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function("part 2 official input with parsing", |b| {
        b.iter(|| {
            let input = black_box(include_str!("../input/day9.in").parse().unwrap());
            day9::process_part1(black_box(&input))
        });
    });
}


pub fn bench_part2_evil1_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function("part 2 evil input 1 with parsing", |b| {
        b.iter(|| {
            let input = include_str!("../test/input/day9_evil1.in").parse().unwrap();
            day9::process_part2(black_box(&input))
        });
    });
}

pub fn bench_part2_evil2_with_parsing(criterion: &mut Criterion) {
    criterion.bench_function("part 2 evil input 2 with parsing", |b| {
        b.iter(|| {
            let input = include_str!("../test/input/day9_evil2.in").parse().unwrap();
            day9::process_part2(black_box(&input))
        });
    });
}

criterion_group!(name = benches;
    config = Criterion::default().with_plots();
    targets = bench_parsing_official, bench_parsing_evil1, bench_parsing_evil2,
    bench_part1_official, bench_part1_evil1, bench_part1_evil2, bench_part2_official, bench_part2_evil1, bench_part2_evil2,
    bench_part1_official_with_parsing, bench_part1_evil1_with_parsing, bench_part1_evil2_with_parsing, bench_part2_official_with_parsing, bench_part2_evil1_with_parsing, bench_part2_evil2_with_parsing,
);
criterion_main!(benches);

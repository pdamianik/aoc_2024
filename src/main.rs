use tokio::join;

use aoc_2024::days;
mod util;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    util::setup()?;

    let (
        day1,
        day2,
        day3,
        day4,
        day5,
        day6,
        day7,
        day8,
        day9,
        day10,
        day11,
        day12,
        day13,
        day14,
        day15,
        day16,
        day17,
    ) = join!(
        tokio::spawn(days::day1::run()),
        tokio::spawn(days::day2::run()),
        tokio::spawn(days::day3::run()),
        tokio::spawn(days::day4::run()),
        tokio::spawn(days::day5::run()),
        tokio::spawn(days::day6::run()),
        tokio::spawn(days::day7::run()),
        tokio::spawn(days::day8::run()),
        tokio::spawn(days::day9::run()),
        tokio::spawn(days::day10::run()),
        tokio::spawn(days::day11::run()),
        tokio::spawn(days::day12::run()),
        tokio::spawn(days::day13::run()),
        tokio::spawn(days::day14::run()),
        tokio::spawn(days::day15::run()),
        tokio::spawn(days::day16::run()),
        tokio::spawn(days::day17::run()),
    );

    day1?
        .and(day2?)
        .and(day3?)
        .and(day4?)
        .and(day5?)
        .and(day6?)
        .and(day7?)
        .and(day8?)
        .and(day9?)
        .and(day10?)
        .and(day11?)
        .and(day12?)
        .and(day13?)
        .and(day14?)
        .and(day15?)
        .and(day16?)
        .and(day17?)
}

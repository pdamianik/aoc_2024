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
}

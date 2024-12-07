use tokio::join;

pub mod days;
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
    ) = join!(
        tokio::spawn(days::day1::run()),
        tokio::spawn(days::day2::run()),
        tokio::spawn(days::day3::run()),
        tokio::spawn(days::day4::run()),
        tokio::spawn(days::day5::run()),
        tokio::spawn(days::day6::run()),
    );

    day1?
        .and(day2?)
        .and(day3?)
        .and(day4?)
        .and(day5?)
        .and(day6?)
}

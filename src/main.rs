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
    ) = join!(
        tokio::spawn(days::day1::run()),
        tokio::spawn(days::day2::run()),
        tokio::spawn(days::day3::run()),
    );

    day1.unwrap()
        .and(day2.unwrap())
        .and(day3.unwrap())
}

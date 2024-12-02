use tokio::join;

pub mod days;
mod util;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    util::setup()?;

    let (
        day1,
        day2
    ) = join!(
        tokio::spawn(days::day1::run()),
        tokio::spawn(days::day2::run())
    );

    day1.unwrap()
        .and(day2.unwrap())
}

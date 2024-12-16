use aoc_2024::days;
mod util;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    util::setup()?;

    days::day16::run().await
}

pub mod days;
mod util;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    util::setup()?;

    days::day3::run().await
}

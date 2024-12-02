use std::str::FromStr;
use std::sync::{Arc, LazyLock};
use eyre::WrapErr;
use reqwest::{Client, Url};
use reqwest::cookie::Jar;

pub mod days;
mod day1;
mod day2;

const CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let jar = Arc::new(Jar::default());
    jar.add_cookie_str(&format!("session={}", std::env::var("AOC_SESSION").unwrap()), &Url::from_str("https://adventofcode.com/").unwrap());
    Client::builder()
        .cookie_store(true)
        .cookie_provider(jar)
        .build().unwrap()
});

fn setup() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt::init();

    Ok(())
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    setup()?;

    days::day1::run().await?;
    days::day2::run().await?;

    Ok(())
}

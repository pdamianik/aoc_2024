use std::str::FromStr;
use std::sync::{Arc, LazyLock};
use reqwest::{Client, Url};
use reqwest::cookie::Jar;

pub const CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let jar = Arc::new(Jar::default());
    jar.add_cookie_str(&format!("session={}", std::env::var("AOC_SESSION").unwrap()), &Url::from_str("https://adventofcode.com/").unwrap());
    Client::builder()
        .cookie_store(true)
        .cookie_provider(jar)
        .build().unwrap()
});

pub fn setup() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt::init();

    Ok(())
}

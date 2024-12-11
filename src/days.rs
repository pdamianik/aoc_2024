use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, LazyLock};
use eyre::{eyre, WrapErr};

use reqwest::header::ACCEPT;
use reqwest::{Client, StatusCode, Url};
use reqwest::cookie::Jar;

pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod day6;
pub mod day7;
pub mod day8;
pub mod day9;
pub mod day10;
pub mod day11;
mod util;

pub const CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let jar = Arc::new(Jar::default());
    jar.add_cookie_str(&format!("session={}", std::env::var("AOC_SESSION").unwrap()), &Url::from_str("https://adventofcode.com/").unwrap());
    Client::builder()
        .cookie_store(true)
        .cookie_provider(jar)
        .build().unwrap()
});

#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
#[repr(transparent)]
pub struct Day(usize);

impl Day {
    pub fn filename(&self) -> String {
        format!("day{}.in", self.0)
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Day {}", self.0)
    }
}

impl FromStr for Day {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<usize>()?.try_into()
    }
}

impl TryFrom<usize> for Day {
    type Error = eyre::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value == 0 || value > 24 {
            Err(eyre!("The day must be between 1 and 24"))
        } else {
            Ok(Day(value))
        }
    }
}

impl Deref for Day {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn get_input(day: Day) -> eyre::Result<String> {
    let input_dir = Path::new("input");
    if !input_dir.exists() {
        std::fs::create_dir(input_dir)
            .wrap_err("Failed to create directory for inputs")?;
    } else if !input_dir.is_dir() {
        return Err(eyre!("./input is not a directory"))
    }

    let input_file = input_dir.join(day.filename());
    let input = std::fs::read_to_string(&input_file);

    if let Ok(input) = input {
        Ok(input)
    } else {
        let response = CLIENT.get(format!("https://adventofcode.com/2024/day/{}/input", *day))
            .header(ACCEPT, "text/plain")
            .send().await
            .context(format!("Failed to request {day} input file"))?;
        let error_message = if response.status() == StatusCode::BAD_REQUEST {
            format!("Failed to request {day} input file. You probably haven't set the AOC_SESSION variable to your session cookie")
        } else {
            format!("Failed to request {day} input file")
        };
        let response = response
            .error_for_status()
            .context(error_message)?;
        let input = response.text().await
            .context(format!("Failed to request {day} input file"))?;
        std::fs::write(&input_file, &input)
            .context(format!("Failed to write input to {}", input_file.display()))?;

        Ok(input)
    }
}

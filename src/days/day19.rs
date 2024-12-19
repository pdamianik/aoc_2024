use std::str::FromStr;
use std::time::SystemTime;
use cached::proc_macro::cached;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(19);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    available_towels: Vec<String>,
    patterns: Vec<String>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (available_towels, orders) = s.split_once("\n\n").unwrap();

        let available_towels = available_towels.trim()
            .split(", ")
            .filter(|towel| !towel.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        let orders = orders.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        Ok(Self {
            available_towels,
            patterns: orders,
        })
    }
}

#[cached(key = "String", convert = r#"{ pattern.to_string() }"#)]
fn count_pattern_combinations(pattern: &str, available_towels: &[String]) -> usize {
    available_towels
        .iter()
        .map(|towel|
            if let Some(rest) = pattern.strip_prefix(towel) {
                if rest.is_empty() {
                    1
                } else {
                    count_pattern_combinations(rest, available_towels)
                }
            } else {
                0
            }
        )
        .sum()
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let result = input.patterns.iter()
        .filter(|order| {
            count_pattern_combinations(order.as_str(), &input.available_towels) > 0
        })
        .count();

    Ok(result)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let result = input.patterns.iter()
        .map(|order| {
            count_pattern_combinations(order.as_str(), &input.available_towels)
        })
        .sum();

    Ok(result)
}

pub async fn run() -> eyre::Result<()> {
    let day_span = span!(Level::ERROR, "", "{}", DAY);
    async {
        info!("Running {DAY}");

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input = raw_input.parse()?;
        debug!(?input);

        let start1 = SystemTime::now();
        let result1 = process_part1(&input)?;
        let end1 = SystemTime::now();
        let start2 = SystemTime::now();
        let result2 = process_part2(&input)?;
        let end2 = SystemTime::now();
        println!("{DAY} result:");
        println!("  part 1: {result1} in {:?}", end1.duration_since(start1).unwrap());
        println!("  part 2: {result2} in {:?}", end2.duration_since(start2).unwrap());
        Ok(())
    }
        .instrument(day_span.or_current())
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    fn example_input() -> Input {
        r"r, wr, b, g, bwu, rb, gb, br

          brwrr
          bggr
          gbbr
          rrbgbr
          ubwu
          bwurrg
          brgr
          bbrgwb
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(6, result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(16, result);
    }
}

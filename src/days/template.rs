use std::ops::{Index, Range};
use std::str::FromStr;
use eyre::{anyhow, WrapErr};
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

const DAY: Day = Day(todo!());

pub fn parse(input: &str) -> eyre::Result<Input> {
    Ok(())
}

type Input = ();

pub fn process_part1(input: &Input) -> eyre::Result<String> {
    let result: usize = todo!();

    Ok(result.to_string())
}

pub fn process_part2(input: &Input) -> eyre::Result<String> {
    let result: usize = todo!();

    Ok(result.to_string())
}

pub async fn run() -> eyre::Result<()> {
    let day_span = span!(Level::ERROR, "", "{}", DAY);
    async {
        info!("Running {DAY}");

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input = parse(&raw_input)?;
        debug!(?input);

        let result1 = process_part1(&input)?;
        // let result2 = process_part2(&input)?;
        println!("{DAY} result:");
        println!("  part 1: {result1}");
        // println!("  part 2: {result2}");
        Ok(())
    }
        .instrument(day_span.or_current())
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_example() {
        let raw_input = todo!() as &str;
        let input = parse(&raw_input).unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!(todo!() as &str, result);

        // let result = process_part2(&input).unwrap();
        // assert_eq!(todo!(), result);
    }
}

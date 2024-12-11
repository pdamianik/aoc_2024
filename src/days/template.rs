use std::str::FromStr;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(todo!());

pub struct Input {

}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let result: usize = todo!();

    Ok(result)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let result: usize = todo!();

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
        // let start2 = SystemTime::now();
        // let result2 = process_part2(&input)?;
        // let end2 = SystemTime::now();
        println!("{DAY} result:");
        println!("  part 1: {result1} in {:?}", end1.duration_since(start1).unwrap());
        // println!("  part 2: {result2} in {:?}", end2.duration_since(start2).unwrap());
        Ok(())
    }
        .instrument(day_span.or_current())
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    fn example_input() -> Input {
        r"".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(todo!() as usize, result);
    }

    #[ignore]
    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(todo!() as usize, result);
    }
}

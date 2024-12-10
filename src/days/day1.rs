use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(1);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Input {
    column1: Vec<usize>,
    column2: Vec<usize>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (first, second) = line.split_once(" ")
                    .ok_or(eyre!("Failed to parse pair of values"))?;
                let first = first.trim().parse()?;
                let second = second.trim().parse()?;
                Ok::<(usize, usize), eyre::Error>((first, second))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (column1, column2) = values.into_iter()
            .unzip();

        Ok(Self {
            column1,
            column2,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<String> {
    let mut column1 = input.column1.clone();
    let mut column2 = input.column2.clone();

    column1.sort();
    column2.sort();

    let result: usize = column1.iter()
        .zip(column2.iter())
        .map(|(column1, column2)| column1.abs_diff(*column2))
        .sum();

    Ok(result.to_string())
}

pub fn process_part2(input: &Input) -> eyre::Result<String> {
    let result: usize = input.column1.iter()
        .map(|&needle| input.column2.iter()
            .filter(|&&val| needle == val)
            .sum::<usize>())
        .sum();


    Ok(result.to_string())
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

    #[test]
    pub fn test_example() {
        let input = r#"3   4
4   3
2   5
1   3
3   9
3   3
"#.parse().unwrap();

        let result1 = process_part1(&input).unwrap();
        assert_eq!("11", result1);

        let result2 = process_part2(&input).unwrap();
        assert_eq!("31", result2);
    }
}

use const_it::slice;
use eyre::eyre;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

const DAY: &str = slice!(file!(), 12..13);

pub fn parse(input: &str) -> eyre::Result<(Vec<usize>, Vec<usize>)> {
    let values = input.lines()
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

    let result = values.into_iter()
        .unzip();

    Ok(result)
}

pub fn process_part1((column1, column2): &(Vec<usize>, Vec<usize>)) -> eyre::Result<String> {
    let mut column1 = column1.clone();
    let mut column2 = column2.clone();

    column1.sort();
    column2.sort();

    let result: usize = column1.iter()
        .zip(column2.iter())
        .map(|(column1, column2)| column1.abs_diff(*column2))
        .sum();

    Ok(result.to_string())
}

pub fn process_part2((column1, column2): &(Vec<usize>, Vec<usize>)) -> eyre::Result<String> {
    let result: usize = column1.iter()
        .map(|&needle| column2.iter()
            .filter(|&&val| needle == val)
            .sum::<usize>())
        .sum();


    Ok(result.to_string())
}

pub async fn run() -> eyre::Result<()> {
    let day: Day = DAY.parse()?;
    let day1_span = span!(Level::INFO, "", "{}", day);
    async {
        info!("Running {day}");

        let raw_input = super::get_input(day).await?;
        trace!(raw_input);

        let input = parse(&raw_input)?;
        debug!(?input);

        let result1 = process_part1(&input)?;
        let result2 = process_part2(&input)?;
        println!("{day} result:");
        println!("  part 1: {result1}");
        println!("  part 2: {result2}");
        Ok(())
    }
        .instrument(day1_span)
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_example() {
        let raw_input = r#"3   4
4   3
2   5
1   3
3   9
3   3
"#;
        let input = parse(&raw_input).unwrap();

        let result1 = process_part1(&input).unwrap();
        assert_eq!("11", result1);

        let result2 = process_part2(&input).unwrap();
        assert_eq!("31", result2);
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    crate::setup()?;
    Ok(())
}
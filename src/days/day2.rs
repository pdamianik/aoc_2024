use std::time::SystemTime;
use eyre::{anyhow, WrapErr};
use itertools::Itertools;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

const DAY: Day = Day(2);

pub fn parse(input: &str) -> eyre::Result<Vec<Vec<usize>>> {
    let values = input.lines()
        .enumerate()
        .map(|(row, line)| {
            line.split_whitespace().into_iter()
                .enumerate()
                .map(|(col, val)|
                    val.parse::<usize>()
                        .wrap_err(anyhow!("Failed to parse {} number in row {row}", col + 1))
                )
                .collect::<Result<Vec<_>, _>>()
        })
        .filter(|line| if let Ok(line) = line { !line.is_empty() } else { true })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(values)
}

fn unsafe_index<'a>(report: impl Iterator<Item = &'a usize>) -> Option<[usize; 4]> {
    let diffs = report
        .enumerate()
        .tuple_windows()
        .map(|((ia, &a), (ib, &b))| {
            let diff = a as isize - b as isize;
            (ia, ib, diff)
        })
        .collect::<Vec<_>>();
    let &(inc_index_a, inc_index_b, _) = diffs.iter()
        .find(|(_, _, diff)| !(1..=3).contains(diff))?;
    let &(dec_index_a, dec_index_b, _) = diffs.iter()
        .find(|(_, _, diff)| !(-3..=-1).contains(diff))?;

    Some([inc_index_a, inc_index_b, dec_index_a, dec_index_b])
}

fn without_index<T>(iter: impl Iterator<Item = T>, index: usize) -> impl Iterator<Item = T> {
    iter.enumerate()
        .filter(move |(i, _)| *i != index)
        .map(|(_, val)| val)
}

fn tolerate(report: &[usize]) -> bool {
    if let Some(unsafe_indices) = unsafe_index(report.iter()) {
        trace!("{unsafe_indices:?}");
        for unsafe_i in unsafe_indices {
            if unsafe_index(without_index(report.iter(), unsafe_i)).is_none() {
                return true;
            }
        }
        false
    } else {
        true
    }
}

pub fn process_part1(reports: &[Vec<usize>]) -> eyre::Result<String> {
    let safe_count = reports.iter()
        .map(|report| unsafe_index(report.iter()).is_none())
        .enumerate()
        .inspect(|&(row, safe)| if safe {
            debug!("Report {row} is safe")
        } else {
            debug!("Report {row} is unsafe")
        })
        .filter(|&(_, safe)| safe)
        .count();

    Ok(safe_count.to_string())
}

pub fn process_part2(reports: &[Vec<usize>]) -> eyre::Result<String> {
    let safe_count = reports.iter()
        .enumerate()
        .map(|(row, report)| {
            trace!("{row}");
            (row, tolerate(report))
        })
        .inspect(|&(row, safe)| if safe {
            debug!("Report {row} is safe")
        } else {
            debug!("Report {row} is unsafe")
        })
        .filter(|&(_, safe)| safe)
        .count();

    Ok(safe_count.to_string())
}

pub async fn run() -> eyre::Result<()> {
    let day_span = span!(Level::ERROR, "", "{}", DAY);
    async {
        info!("Running {DAY}");

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input = parse(&raw_input)?;
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
        let raw_input = r#"
        7 6 4 2 1
        1 2 7 8 9
        9 7 6 2 1
        1 3 2 4 5
        8 6 4 4 1
        1 3 6 7 9
        "#;
        let input = parse(&raw_input).unwrap();

        let result1 = process_part1(&input).unwrap();
        assert_eq!("2", result1);

        let result2 = process_part2(&input).unwrap();
        assert_eq!("4", result2);
    }
}

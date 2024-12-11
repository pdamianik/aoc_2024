use std::collections::HashMap;
use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
use itertools::Itertools;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(11);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Stone {
    engraving: usize,
}

impl Stone {
    pub fn evolve(&mut self, cache: &mut HashMap<Stone, (Stone, Option<Stone>)>) -> Option<Self> {
        let (stone, new_stone) = cache.entry(*self).or_insert_with(|| {
            match self.engraving {
                0 => (Stone { engraving: 1 }, None),
                engraving if engraving.ilog10() % 2 == 1 => {
                    let half = 10usize.pow(engraving.ilog10() / 2 + 1);
                    (
                        Stone { engraving: engraving / half },
                        Some(Self {
                            engraving: engraving % half,
                        })
                    )
                },
                engraving => {
                    (Stone { engraving: engraving * 2024 }, None)
                }
            }
        });
        *self = *stone;
        *new_stone
    }
}

impl FromStr for Stone {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engraving = s.parse()?;
        Ok(Self {
            engraving,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    stones: Vec<Stone>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.lines()
            .map(|line| line.trim())
            .find(|line| !line.is_empty())
            .ok_or(eyre!("Input is empty"))?;

        let stones = input.split(" ")
            .map(Stone::from_str)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            stones,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let mut stones = input.stones.clone();
    let mut cache: HashMap<Stone, (Stone, Option<Stone>)> = HashMap::new();

    let mut acc = Vec::new();
    for _ in 0..25 {
        for stone in &mut stones {
            if let Some(new_stone) = stone.evolve(&mut cache) {
                acc.push(new_stone);
            }
        }

        stones.append(&mut acc);
    }

    Ok(stones.len())
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let mut stones: HashMap<Stone, usize> = input.stones.iter().cloned().counts();
    let mut new_stones: HashMap<Stone, usize> = HashMap::with_capacity(stones.len());
    let mut cache: HashMap<Stone, (Stone, Option<Stone>)> = HashMap::new();

    for _ in 0..75 {
        for (stone, &count) in &stones {
            let mut stone = *stone;
            if let Some(new_stone) = stone.evolve(&mut cache) {
                insert_stone_count(&mut new_stones, count, new_stone);
            }
            insert_stone_count(&mut new_stones, count, stone);
        }

        std::mem::swap(&mut stones, &mut new_stones);
        new_stones.clear();
    }

    Ok(stones.values().sum::<usize>())
}

fn insert_stone_count(new_stones: &mut HashMap<Stone, usize>, count: usize, stone: Stone) {
    new_stones.entry(stone)
        .and_modify(|saved_count| *saved_count += count)
        .or_insert(count);
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
        r"125 17".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(55312, result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(65601038650482, result);
    }
}

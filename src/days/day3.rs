use std::ops::{Index, Range};
use std::str::FromStr;
use std::time::SystemTime;
use eyre::{anyhow, WrapErr};
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

const DAY: Day = Day(3);

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Instruction {
    Mul(u16, u16),
    Do,
    Dont,
    Noop,
}

impl Instruction {
    pub fn len(&self) -> usize {
        match self {
            // `mul` (3) + `(` (1) + `[a]` + `,` (1) + `[b]` + `)` (1)
            Self::Mul(a, b) => "mul".len() + 1 + a.ilog10() as usize + 1 + b.ilog10() as usize + 1,
            // `do` (2) + `(` (1) + `)` (1)
            Self::Do => "do".len() + 1 + 1,
            // `don't` (5) + `(` (1) + `)` (1)
            Self::Dont => "don't".len() + 1 + 1,
            Self::Noop => 0,
        }
    }

    pub fn is_noop(&self) -> bool {
        if let Self::Noop = self {
            true
        } else {
            false
        }
    }
}

fn argument_ranges(s: &str) -> Option<Vec<Range<usize>>> {
    let closing_bracket = s.find(')')?;
    let s = &s[..closing_bracket];
    let comma_indices = s.chars()
        .enumerate()
        .filter_map(|(i, character)|
            if character == ',' {
                Some(i)
            } else {
                None
            }
        );
    let ranges = comma_indices.scan(0, |start, comma| {
        let range = Some(*start..comma);
        *start = comma + 1;
        range
    });
    let last_comma = s.rfind(',').unwrap_or(0);
    let ranges = ranges
        .chain(std::iter::once(last_comma + 1..s.len()))
        .collect();
    Some(ranges)
}

impl FromStr for Instruction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("mul(") {
            let s = &s[4..];
            let argument_ranges = argument_ranges(s)
                .ok_or(anyhow!("No closing bracket found "))?;

            if argument_ranges.len() != 2 {
                return Err(anyhow!("Expected 2 arguments for `mul(a, b)`"))
            }

            let arguments = argument_ranges.into_iter()
                .map(|range|
                    if range.len() > 3 {
                        Err(anyhow!("`mul(a, b)` argument is longer than 3 characters"))
                    } else {
                        s.index(range.clone()).parse()
                            .wrap_err(format!("failed to parse mul(a, b) argument {}", &s[range]))
                    }
                )
                .collect::<Result<Vec<u16>, _>>()?;

            Ok(Self::Mul(arguments[0], arguments[1]))
        } else if s.starts_with("do()") {
            Ok(Self::Do)
        } else if s.starts_with("don't()") {
            Ok(Self::Dont)
        } else {
            Err(anyhow!("Invalid instruction"))
        }
    }
}

pub fn parse(input: &str) -> eyre::Result<Input> {
    let mut instructions = Vec::new();
    let mut start = 0;
    while start < input.len() {
        match input[start..].parse::<Instruction>() {
            Ok(instruction) => {
                debug!(?instruction);
                start += instruction.len();
                instructions.push(instruction);
            },
            Err(err) => {
                start += 1;
                trace!(?err)
            },
        }
    }

    Ok(instructions)
}

type Input = Vec<Instruction>;

pub fn process_part1(input: &Input) -> eyre::Result<String> {
    let result = input.iter()
        .filter_map(|instruction|
            match instruction {
                &Instruction::Mul(a, b) => Some(a as usize * b as usize),
                _ => None,
            }
)
        .sum::<usize>();

    Ok(result.to_string())
}

pub fn process_part2(input: &Input) -> eyre::Result<String> {
    let filtered_instructions = input.iter()
        .scan(true, |execute, instruction| {
            match instruction {
                Instruction::Mul(_, _) if *execute => Some(instruction),
                Instruction::Do => {
                    *execute = true;
                    Some(&Instruction::Noop)
                },
                Instruction::Dont => {
                    *execute = false;
                    Some(&Instruction::Noop)
                },
                _ => Some(&Instruction::Noop),
            }
        })
        .filter(|instruction| !instruction.is_noop())
        .cloned()
        .collect();
    debug!(?filtered_instructions);

    let result = process_part1(&filtered_instructions)?;

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
        let raw_input = r#"xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))"#;
        let input = parse(&raw_input).unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!("161", result);

        let raw_input = r#"xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))"#;
        let input = parse(&raw_input).unwrap();

        let result = process_part2(&input).unwrap();
        assert_eq!("48", result);
    }
}

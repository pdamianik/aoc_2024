use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
use eyre::anyhow;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::Lines;

const DAY: Day = Day(7);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Operator {
    Add,
    Multiply,
    Concatenate,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Add => "+",
            Self::Multiply => "*",
            Self::Concatenate => "||",
        })
    }
}

impl Default for Operator {
    fn default() -> Self {
        Self::Add
    }
}

impl Operator {
    const ALL1: [Self; 2] = [Self::Add, Self::Multiply];
    const ALL2: [Self; 3] = [Self::Add, Self::Multiply, Self::Concatenate];

    pub fn apply(&self, a: usize, b: usize) -> usize {
        match self {
            Self::Add => a.checked_add(b).unwrap(),
            Self::Multiply => a.checked_mul(b).unwrap(),
            Self::Concatenate => a.checked_mul(10usize.checked_pow(b.checked_ilog10().unwrap().checked_add(1).unwrap()).unwrap()).unwrap().checked_add(b).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Equation {
    result: usize,
    operands: VecDeque<usize>,
}

impl Equation {
    pub fn try_solve(&mut self, operators: &[Operator]) -> bool {
        let mut equations = VecDeque::from_iter(std::iter::once(self.clone()));
        while let Some(mut equation) = equations.pop_front() {
            let first = equation.operands.pop_front().unwrap();
            let second = *equation.operands.front().unwrap();

            for operator in operators {
                let preliminary_result = operator.apply(first, second);
                match preliminary_result.cmp(&equation.result) {
                    Ordering::Less => if equation.operands.len() == 1 {
                        continue;
                    },
                    Ordering::Equal if equation.operands.len() == 1 => return true,
                    Ordering::Equal => (),
                    Ordering::Greater => continue,
                }
                equation.operands[0] = preliminary_result;
                equations.push_back(equation.clone());
            }
        }
        false
    }
}

impl FromStr for Equation {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (result, operands) = s.split_once(':')
            .ok_or(anyhow!("Failed to split result from operands"))?;
        let result = result.parse()?;
        let operands = operands.split(' ')
            .filter(|operand| !operand.is_empty())
            .map(|operand| operand.parse::<usize>())
            .collect::<Result<VecDeque<_>, _>>()?;
        if operands.len() < 2 {
            return Err(anyhow!("Could not find two or more operand for equation"));
        }
        Ok(Self {
            result,
            operands,
        })
    }
}

type Input = Lines<Equation>;

pub async fn process_part1(input: &Input) -> eyre::Result<(usize, Vec<Equation>)> {
    let handles = input.iter()
        .map(|equation| {
            let mut equation = equation.clone();
            tokio::spawn(async move { (equation.try_solve(&Operator::ALL1), equation) })
            // (equation.try_solve(&Operator::ALL1), equation)
        })
        .collect::<Vec<_>>();
    let mut result = 0;
    let mut failed = Vec::new();
    for handle in handles {
        let (solved, equation) = handle.await?;
        // let (solved, equation) = handle;
        if solved {
            result += equation.result;
        } else {
            failed.push(equation)
        }
    }

    Ok((result, failed))
}

pub async fn process_part2(input: &[Equation], part1: usize) -> eyre::Result<usize> {
    let handles = input.iter()
        .map(|equation| {
            let mut equation = equation.clone();
            tokio::spawn(async move { (equation.try_solve(&Operator::ALL2), equation) })
            // (equation.try_solve(&Operator::ALL2), equation)
        })
        .collect::<Vec<_>>();
    let mut result = part1;
    for handle in handles {
        let (solved, equation) = handle.await?;
        // let (solved, equation) = handle;
        if solved {
            result += equation.result;
        }
    }

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
        let (result1, failed) = process_part1(&input).await?;
        let end1 = SystemTime::now();
        let start2 = SystemTime::now();
        let result2 = process_part2(&failed, result1).await?;
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
        r"190: 10 19
          3267: 81 40 27
          83: 17 5
          156: 15 6
          7290: 6 8 6 15
          161011: 16 10 13
          192: 17 8 14
          21037: 9 7 18 13
          292: 11 6 16 20
          ".parse().unwrap()
    }

    #[tokio::test]
    pub async fn test_part1() {
        let input = example_input();
        println!("{input:?}");

        let (result, _) = process_part1(&input).await.unwrap();
        assert_eq!(3749, result);
    }

    #[tokio::test]
    pub async fn test_part1_custom() {
        let raw_input = r"3744: 9 7 18 13
                               104831: 9 7 18 13 4 7
                               104832: 9 7 18 13 4 7
                               ";
        let input: Input = raw_input.parse().unwrap();

        let (result, _) = process_part1(&input).await.unwrap();
        assert_eq!(108576, result);
    }

    #[test]
    pub fn test_concat() {
        assert_eq!(Operator::Concatenate.apply(2, 1), 21);
        assert_eq!(Operator::Concatenate.apply(327, 934), 327934);
        assert_eq!(Operator::Concatenate.apply(12, 345), 12345);
        assert_eq!(Operator::Concatenate.apply(1200, 345), 1200345);

        for a in 1..100 {
            for b in 1..10000 {
                assert_eq!(Operator::Concatenate.apply(a, b), format!("{a}{b}").parse::<usize>().unwrap());
            }
        }
    }

    #[tokio::test]
    pub async fn test_part2() {
        let input: Input = example_input();

        let (result, failed) = process_part1(&input).await.unwrap();
        let result = process_part2(&failed, result).await.unwrap();
        assert_eq!(11387, result);
    }
}

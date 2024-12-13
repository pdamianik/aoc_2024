use std::convert::identity;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(13);

pub trait ButtonType {
    const COST: usize;
    const LABEL: &'static str;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct A;

impl ButtonType for A {
    const COST: usize = 3;
    const LABEL: &'static str = "A";
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct B;

impl ButtonType for B {
    const COST: usize = 1;
    const LABEL: &'static str = "B";
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Button<Type: ButtonType> {
    x: usize,
    y: usize,
    button_type: PhantomData<Type>,
}

impl<Type: ButtonType> Button<Type> {
    pub const fn cost() -> usize {
        Type::COST
    }

    pub const fn label() -> &'static str {
        Type::LABEL
    }
}

impl<Type: ButtonType> FromStr for Button<Type> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (button_type, distance) = s.strip_prefix("Button ")
            .ok_or(eyre!("Expected button line to start with \"Button [type]:\""))?
            .split_once(": ")
            .ok_or(eyre!("Expected button line to start with \"Button [type]:\""))?;

        if button_type != Type::LABEL {
            return Err(eyre!("Button type {button_type} should be {}", Type::LABEL))
        }
        let (x, y) = distance.split_once(", ")
            .ok_or(eyre!("Failed to split button distance"))?;
        let x = x.strip_prefix("X+")
            .ok_or(eyre!("A buttons x distance should be given with \"X+\""))?
            .parse()?;
        let y = y.strip_prefix("Y+")
            .ok_or(eyre!("A buttons y distance should be given with \"Y+\""))?
            .parse()?;

        Ok(Self { x, y, button_type: PhantomData })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClawMachine {
    button_a: Button<A>,
    button_b: Button<B>,
    x: usize,
    y: usize,
}

impl ClawMachine {
    pub fn solve(&self) -> Option<(usize, usize)> {
        let b_top = self.x as isize * self.button_a.y as isize - self.button_a.x as isize * self.y as isize;
        let b_bottom = self.button_b.x as isize * self.button_a.y as isize - self.button_a.x as isize * self.button_b.y as isize;

        if b_top % b_bottom != 0 {
            return None;
        };

        let b= b_top / b_bottom;
        let a_top = self.y as isize - b * self.button_b.y as isize;
        let a_bottom = self.button_a.y as isize;

        if a_top % a_bottom != 0 {
            return None;
        };

        let a = a_top / a_bottom;
        Some((a as usize, b as usize))
    }

    pub fn cost(&self) -> Option<usize> {
        let (a, b) = self.solve()?;
        Some(a * A::COST + b * B::COST)
    }
}

impl FromStr for ClawMachine {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines()
            .map(|line| line.trim());
        let button_a = lines.next()
            .ok_or(eyre!("Failed to parse the first button"))?
            .parse()?;
        let button2 = lines.next()
            .ok_or(eyre!("Failed to parse the second button"))?
            .parse()?;
        let prize = lines.next()
            .ok_or(eyre!("Failed to parse the price"))?;


        let location =  prize.strip_prefix("Prize: ")
            .ok_or(eyre!("Expected \"Prize: \" label in the third row"))?;
        let (x, y) = location.split_once(", ")
            .ok_or(eyre!("Failed to split price x and y location"))?;
        let x = x.strip_prefix("X=")
            .ok_or(eyre!("price x location should be given with \"X=\""))?
            .parse()?;
        let y = y.strip_prefix("Y=")
            .ok_or(eyre!("price y location should be given with \"Y=\""))?
            .parse()?;

        Ok(Self {
            button_a,
            button_b: button2,
            x, y,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    claw_machines: Vec<ClawMachine>
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let claw_machines = s.split("\n\n")
            .map(ClawMachine::from_str)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            claw_machines,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let result = input.claw_machines.iter()
        .map(ClawMachine::cost)
        .filter_map(identity)
        .sum();

    Ok(result)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let claw_machines = input.claw_machines.iter()
        .map(|claw_machine| {
            let mut claw_machine = claw_machine.clone();
            claw_machine.x += 10000000000000;
            claw_machine.y += 10000000000000;
            claw_machine
        })
        .collect::<Vec<_>>();
    let result = claw_machines.iter()
        .map(ClawMachine::cost)
        .filter_map(identity)
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
        r"Button A: X+94, Y+34
          Button B: X+22, Y+67
          Prize: X=8400, Y=5400

          Button A: X+26, Y+66
          Button B: X+67, Y+21
          Prize: X=12748, Y=12176

          Button A: X+17, Y+86
          Button B: X+84, Y+37
          Prize: X=7870, Y=6450

          Button A: X+69, Y+23
          Button B: X+27, Y+71
          Prize: X=18641, Y=10279".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(480, result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(875318608908, result);
    }
}

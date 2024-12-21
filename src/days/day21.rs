use std::convert::identity;
use std::marker::PhantomData;
use std::str::{Chars, FromStr};
use std::time::SystemTime;
use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::Coordinate;

pub const DAY: Day = Day(21);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    codes: Vec<String>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let codes = s.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|code| code.to_string())
            .collect();

        Ok(Self {
            codes,
        })
    }
}

pub trait Priority {
    fn step_in_direction(&self, direction: Coordinate) -> char;
}

const PRIORITIES: [&dyn Priority; 2] = [&HorizontalPriority, &VerticalPriority];

#[derive(Copy, Clone, Default)]
pub struct HorizontalPriority;

impl Priority for HorizontalPriority {
    fn step_in_direction(&self, direction: Coordinate) -> char {
        match direction {
            Coordinate(1, _) => '>',
            Coordinate(-1, _) => '<',
            Coordinate(0, 1) => 'v',
            Coordinate(0, -1) => '^',
            _ => panic!(),
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct VerticalPriority;

impl Priority for VerticalPriority {
    fn step_in_direction(&self, direction: Coordinate) -> char {
        match direction {
            Coordinate(_, 1) => 'v',
            Coordinate(_, -1) => '^',
            Coordinate(1, 0) => '>',
            Coordinate(-1, 0) => '<',
            _ => panic!(),
        }
    }
}

pub trait Keypad {
    const START: Coordinate;

    fn symbol_to_coordinate(symbol: char) -> Coordinate;

    fn coordinate_to_symbol(coordinate: Coordinate) -> char;
}

pub struct NumericKeypad<Source: Iterator<Item = char>> {
    to_type: Source,
    current: Coordinate,
    target: Coordinate,
    finished: bool,
    priority: &'static dyn Priority,
}

impl<S: Iterator<Item = char>> Keypad for NumericKeypad<S> {
    const START: Coordinate = Coordinate(2, 3);

    fn symbol_to_coordinate(symbol: char) -> Coordinate {
        match symbol {
            'A' => Coordinate(2, 3),
            '0' => Coordinate(1, 3),
            next if next >= '1' && next <= '9' => {
                let next = next as isize - '1' as isize;
                Coordinate(next % 3, 2 - next / 3)
            }
            _ => panic!("invalid directional keypad target {symbol}")
        }
    }

    fn coordinate_to_symbol(coordinate: Coordinate) -> char {
        match coordinate {
            Coordinate(x, 0) => ['7', '8', '9'][x as usize],
            Coordinate(x, 1) => ['4', '5', '6'][x as usize],
            Coordinate(x, 2) => ['1', '2', '3'][x as usize],
            Coordinate(x, 3) if x > 0 => [' ', '0', 'A'][x as usize],
            _ => panic!("Invalid directional keypad position {coordinate}")
        }
    }
}

impl<Source: Iterator<Item = char>> NumericKeypad<Source> {
    pub fn new(mut to_type: Source) -> Self {
        let first = to_type.next().unwrap();
        Self {
            to_type,
            current: Self::START,
            target: Self::symbol_to_coordinate(first),
            finished: false,
        }
    }
}

impl<S: Iterator<Item = char>> Iterator for NumericKeypad<S> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let direction = (self.target - self.current).eigen_axis();
        if direction == Coordinate(0, 0) {
            if let Some(next) = self.to_type.next() {
                self.target = Self::symbol_to_coordinate(next);
            } else {
                self.finished = true;
            }

            return Some('A');
        }
        let step = match (self.current, self.target) {
            (Coordinate(0, _), Coordinate(_, 3)) => '>',
            (Coordinate(_, 3), Coordinate(0, _)) => '^',
            _ => self.priority.step_in_direction(direction),
        };
        match step {
            '^' => self.current.1 -= 1,
            '>' => self.current.0 += 1,
            'v' => self.current.1 += 1,
            '<' => self.current.0 -= 1,
            _ => unreachable!()
        }
        Some(step)
    }
}

pub struct DirectionalKeypad<Source: Iterator<Item = char>> {
    to_type: Source,
    current: Coordinate,
    target: Coordinate,
    finished: bool,
    priority: &'static dyn Priority,
}

impl<S: Iterator<Item = char>> Keypad for DirectionalKeypad<S> {
    const START: Coordinate = Coordinate(2, 0);

    fn symbol_to_coordinate(symbol: char) -> Coordinate {
        match symbol {
            '^' => Coordinate(1, 0),
            'A' => Coordinate(2, 0),
            '<' => Coordinate(0, 1),
            'v' => Coordinate(1, 1),
            '>' => Coordinate(2, 1),
            _ => panic!("Invalid directional keypad target {symbol}"),
        }
    }

    fn coordinate_to_symbol(coordinate: Coordinate) -> char {
        match coordinate {
            Coordinate(1, 0) => '^',
            Coordinate(2, 0) => 'A',
            Coordinate(0, 1) => '<',
            Coordinate(1, 1) => 'v',
            Coordinate(2, 1) => '>',
            _ => panic!("Invalid directional keypad position {coordinate}")
        }
    }
}

impl<Source: Iterator<Item = char>> DirectionalKeypad<Source> {
    pub fn new(mut to_type: Source, priority: &'static dyn Priority) -> Self {
        let first = to_type.next().unwrap();
        Self {
            to_type,
            current: Self::START,
            target: Self::symbol_to_coordinate(first),
            finished: false,
            priority,
        }
    }
}


impl<S: Iterator<Item = char>> Iterator for DirectionalKeypad<S> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let direction = (self.target - self.current).eigen_axis();
        if direction == Coordinate(0, 0) {
            if let Some(next) = self.to_type.next() {
                self.target = Self::symbol_to_coordinate(next);
            } else {
                self.finished = true;
            }

            return Some('A');
        }
        let step = match (self.current, self.target) {
            (Coordinate(0, _), Coordinate(_, 0)) => '>',
            (Coordinate(_, 0), Coordinate(0, _)) => 'v',
            _ => self.priority.step_in_direction(direction)
        };
        match step {
            '^' => self.current.1 -= 1,
            '>' => self.current.0 += 1,
            'v' => self.current.1 += 1,
            '<' => self.current.0 -= 1,
            _ => unreachable!()
        }
        Some(step)
    }
}

pub struct Simulate<Steps: Iterator<Item = char>, K: Keypad> {
    steps: Steps,
    position: Coordinate,
    keypad: PhantomData<K>,
}

impl<Steps: Iterator<Item = char>, K: Keypad> Simulate<Steps, K> {
    pub fn new(steps: Steps) -> Self {
        Self {
            steps,
            position: K::START,
            keypad: PhantomData,
        }
    }
}

impl<S: Iterator<Item = char>, K: Keypad> Iterator for Simulate<S, K> {
    type Item = Option<char>;

    fn next(&mut self) -> Option<Self::Item> {
        let step = self.steps.next()?;
        match step {
            '^' => self.position.1 -= 1,
            '>' => self.position.0 += 1,
            'v' => self.position.1 += 1,
            '<' => self.position.0 -= 1,
            'A' => return Some(Some(K::coordinate_to_symbol(self.position))),
            _ => panic!("Invalid step {step}"),
        }
        Some(None)
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let result = input.codes.iter()
        // .zip([
        //     "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A",
        //     "<v<A>>^AAAvA^A<vA<AA>>^AvAA<^A>A<v<A>A>^AAAvA<^A>A<vA>^A<A>A",
        //     "<v<A>>^A<vA<A>>^AAvAA<^A>A<v<A>>^AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A",
        //     "<v<A>>^AA<vA<A>>^AAvAA<^A>A<vA>^A<A>A<vA>^A<A>A<v<A>A>^AAvA<^A>A",
        //     "<v<A>>^AvA^A<vA<AA>>^AAvA<^A>AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A",
        // ])
        .map(|code| {
            let code_num: usize = code[0..code.len() - 1].parse().unwrap();

            let mut current: Vec<String> = PRIORITIES.iter()
                .map(|priority| NumericKeypad::new(code.chars(), *priority).collect())
                .collect();
            for _ in 0..2 {
                let next: Vec<String> = current.iter()
                    .flat_map(|steps| PRIORITIES.iter()
                        .map(|priority| DirectionalKeypad::new(steps.chars(), *priority).collect())
                    )
                    .collect();
                let min_length = next.iter().map(String::len).min().unwrap();
                current.clear();
                current.extend(next.into_iter()
                    .filter(|steps| steps.len() == min_length)
                )
            }
            // println!("{code}: {} {}", current.first().unwrap().chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>(), current.first().unwrap().len());
            // println!("{code}: {} {}", solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>(), solution.len());
            // let simulated = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(steps.chars()).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity).collect::<String>();
            // println!("{code}: simulated once {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated once {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // let simulated = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(steps.chars()).filter_map(identity)).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity)).filter_map(identity).collect::<String>();
            // println!("{code}: simulated twice {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated twice {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            code_num * current.first().unwrap().len()
        })
        .sum();

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
        r"029A
          980A
          179A
          456A
          379A
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(126384, result);
    }

    #[ignore]
    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(todo!() as usize, result);
    }
}

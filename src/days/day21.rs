use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::convert::identity;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ops::{BitAnd, Index, Mul};
use std::path::Iter;
use std::rc::Rc;
use std::str::{Chars, FromStr};
use std::time::SystemTime;
use std::vec::IntoIter;
use cached::UnboundCache;
use itertools::Itertools;
use num_bigint::{BigUint, ToBigUint};
use num_traits::One;
use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Coordinate, Direction};

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

pub trait Keypad<S: Iterator<Item = Self::Output>>: Iterator<Item = Route> {
    const START: Coordinate;

    type Output;

    fn input_to_coordinate(input: char) -> Coordinate;

    fn coordinate_to_input(coordinate: Coordinate) -> char;

    fn route_to_coordinate(from: Coordinate, to: Coordinate) -> Route;

    fn new(to_type: S) -> Self;
}

pub struct NumericKeypad<Source: Iterator<Item=char>> {
    to_type: Source,
    current: Coordinate,
}

impl<S: Iterator<Item=char>> Keypad<S> for NumericKeypad<S> {
    const START: Coordinate = Coordinate(2, 3);
    type Output = char;

    fn input_to_coordinate(input: char) -> Coordinate {
        match input {
            'A' => Coordinate(2, 3),
            '0' => Coordinate(1, 3),
            next if next >= '1' && next <= '9' => {
                let next = next as isize - '1' as isize;
                Coordinate(next % 3, 2 - next / 3)
            }
            _ => panic!("Invalid directional keypad target {input}")
        }
    }

    fn coordinate_to_input(coordinate: Coordinate) -> char {
        if coordinate.0 < 0 || coordinate.0 > 2 || coordinate.1 < 0 || coordinate.1 > 3 || coordinate.0 == 0 && coordinate.1 == 3 {
            panic!("Invalid directional keypad position {coordinate}")
        }
        [
            ['7', '8', '9'],
            ['4', '5', '6'],
            ['1', '2', '3'],
            [' ', '0', 'A'],
        ][coordinate.1 as usize][coordinate.0 as usize]
    }

    fn route_to_coordinate(from: Coordinate, to: Coordinate) -> Route {
        let distance = to - from;

        match (from, to) {
            (Coordinate(0, _), Coordinate(_, 3)) => Route::Segmented(Leg(Direction::East, distance.0.abs() as usize), Leg(Direction::South, distance.1.abs() as usize), false),
            (Coordinate(_, 3), Coordinate(0, _)) => Route::Segmented(Leg(Direction::North, distance.1.abs() as usize), Leg(Direction::West, distance.0.abs() as usize), false),
            _ => match (distance.0, distance.1) {
                (0, 0) => Route::Empty(1),
                (x, 0) if x > 0 => Route::Direct(Leg(Direction::East, x as usize)),
                (x, 0) if x < 0 => Route::Direct(Leg(Direction::West, -x as usize)),
                (0, y) if y > 0 => Route::Direct(Leg(Direction::South, y as usize)),
                (0, y) if y < 0 => Route::Direct(Leg(Direction::North, -y as usize)),
                (x, y) => {
                    let x_leg = match x {
                        x if x < 0 => Leg(Direction::West, -x as usize),
                        x if x > 0 => Leg(Direction::East, x as usize),
                        _ => unreachable!()
                    };
                    let y_leg = match y {
                        y if y < 0 => Leg(Direction::North, -y as usize),
                        y if y > 0 => Leg(Direction::South, y as usize),
                        _ => unreachable!()
                    };
                    Route::Segmented(x_leg, y_leg, true)
                }
            },
        }
    }

    fn new(to_type: S) -> Self {
        Self {
            to_type,
            current: Self::START,
        }
    }
}

impl<S: Iterator<Item=char>> Iterator for NumericKeypad<S> {
    type Item = Route;

    fn next(&mut self) -> Option<Self::Item> {
        let target = Self::input_to_coordinate(self.to_type.next()?);
        let route = Self::route_to_coordinate(self.current, target);
        self.current = target;
        Some(route)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Leg(Direction, usize);

impl PartialOrd for Leg {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(self.1.cmp(&other.1))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd)]
pub enum Route {
    Empty(usize),
    Direct(Leg),
    Segmented(Leg, Leg, bool),
}

impl Route {
    pub fn chars(&self) -> RouteChars {
        RouteChars::new(self.clone())
    }

    pub const fn len(&self) -> usize {
        match self {
            Self::Empty(count) => *count,
            Self::Direct(Leg(_, distance)) => *distance + 1,
            Self::Segmented(
                Leg(_, first_distance),
                Leg(_, second_distance),
                _,
            ) => *first_distance + *second_distance + 1,
        }
    }

    pub const fn reversible(&self) -> bool {
        match self {
            Route::Segmented(_, _, reversible) => *reversible,
            _ => false
        }
    }

    pub fn reverse(&self) -> Self {
        match self {
            Self::Segmented(first, second, true) => Self::Segmented(second.clone(), first.clone(), true),
            _ => panic!(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RouteChars {
    route: Route,
    finished: bool,
}

impl RouteChars {
    pub fn new(route: Route) -> Self {
        Self { route, finished: false }
    }
}

impl Iterator for RouteChars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        match &mut self.route {
            Route::Empty(count) if *count != 0 => {
                *count -= 1;
                Some('A')
            }
            Route::Empty(_) => {
                self.finished = true;
                None
            }
            Route::Direct(Leg(direction, distance)) if *distance != 0 => {
                *distance -= 1;
                Some(direction.symbol())
            }
            Route::Direct(_) => {
                self.finished = true;
                Some('A')
            }
            Route::Segmented(
                Leg(direction, distance),
                _,
                _,
            ) if *distance != 0 => {
                *distance -= 1;
                Some(direction.symbol())
            }
            Route::Segmented(
                _,
                Leg(direction, distance),
                _,
            ) if *distance != 0 => {
                *distance -= 1;
                Some(direction.symbol())
            }
            Route::Segmented(_, _, _) => {
                self.finished = true;
                Some('A')
            }
        }
    }
}

pub struct DirectionalKeypad<Source: Iterator<Item=Route>> {
    to_type: Source,
    queue: Vec<Route>,
}

impl<S: Iterator<Item=Route>> Keypad<S> for DirectionalKeypad<S> {
    const START: Coordinate = Coordinate(2, 0);

    type Output = Route;

    fn input_to_coordinate(input: char) -> Coordinate {
        match input {
            '^' => Coordinate(1, 0),
            'A' => Coordinate(2, 0),
            '<' => Coordinate(0, 1),
            'v' => Coordinate(1, 1),
            '>' => Coordinate(2, 1),
            _ => panic!("Invalid directional keypad target {input}"),
        }
    }

    fn coordinate_to_input(coordinate: Coordinate) -> char {
        match coordinate {
            Coordinate(1, 0) => '^',
            Coordinate(2, 0) => 'A',
            Coordinate(0, 1) => '<',
            Coordinate(1, 1) => 'v',
            Coordinate(2, 1) => '>',
            _ => panic!("Invalid directional keypad position {coordinate}")
        }
    }

    fn route_to_coordinate(from: Coordinate, to: Coordinate) -> Route {
        let distance = to - from;

        match (from, to) {
            (Coordinate(0, _), Coordinate(_, 0)) => Route::Segmented(Leg(Direction::East, distance.0.abs() as usize), Leg(Direction::North, distance.1.abs() as usize), false),
            (Coordinate(_, 0), Coordinate(0, _)) => Route::Segmented(Leg(Direction::South, distance.1.abs() as usize), Leg(Direction::West, distance.0.abs() as usize), false),
            _ => match (distance.0, distance.1) {
                (0, 0) => Route::Empty(1),
                (x, 0) if x > 0 => Route::Direct(Leg(Direction::East, x as usize)),
                (x, 0) if x < 0 => Route::Direct(Leg(Direction::West, -x as usize)),
                (0, y) if y > 0 => Route::Direct(Leg(Direction::South, y as usize)),
                (0, y) if y < 0 => Route::Direct(Leg(Direction::North, -y as usize)),
                (x, y) => {
                    let x_leg = match x {
                        x if x < 0 => Leg(Direction::West, -x as usize),
                        x if x > 0 => Leg(Direction::East, x as usize),
                        _ => unreachable!()
                    };
                    let y_leg = match y {
                        y if y < 0 => Leg(Direction::North, -y as usize),
                        y if y > 0 => Leg(Direction::South, y as usize),
                        _ => unreachable!()
                    };
                    Route::Segmented(x_leg, y_leg, true)
                }
            },
        }
    }

    fn new(to_type: S) -> Self {
        Self {
            to_type,
            queue: Vec::new(),
        }
    }
}

impl<S: Iterator<Item=Route>> Iterator for DirectionalKeypad<S> {
    type Item = Route;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.queue.pop() {
            return Some(item);
        }
        let next = self.to_type.next()?;
        match next {
            Route::Empty(_) => return Some(next),
            Route::Direct(Leg(direction, distance)) => {
                let target = Self::input_to_coordinate(direction.symbol());
                let route = Self::route_to_coordinate(Self::START, target);
                let back = Self::route_to_coordinate(target, Self::START);
                self.queue.reserve(3);
                self.queue.push(back);
                if distance > 1 {
                    self.queue.push(Route::Empty(distance - 1));
                }
                self.queue.push(route);
            }
            Route::Segmented(
                Leg(direction1, distance1),
                Leg(direction2, distance2),
                _,
            ) => {
                let target1 = Self::input_to_coordinate(direction1.symbol());
                let target2 = Self::input_to_coordinate(direction2.symbol());
                let route1  = Self::route_to_coordinate(Self::START, target1);
                let route2 = Self::route_to_coordinate(target1, target2);
                let back = Self::route_to_coordinate(target2, Self::START);
                self.queue.reserve(5);
                self.queue.push(back);
                if distance2 > 1 {
                    self.queue.push(Route::Empty(distance2 - 1))
                }
                self.queue.push(route2);
                if distance1 > 1 {
                    self.queue.push(Route::Empty(distance1 - 1))
                }
                self.queue.push(route1);
            }
        }

        self.queue.pop()
    }
}

// pub struct Simulate<Steps: Iterator<Item=char>, K: Keypad<Steps, Output = char>> {
//     steps: Steps,
//     position: Coordinate,
//     keypad: PhantomData<K>,
// }
//
// impl<Steps: Iterator<Item=char>, K: Keypad<Steps>> Simulate<Steps, K> {
//     pub fn new(steps: Steps) -> Self {
//         Self {
//             steps,
//             position: K::START,
//             keypad: PhantomData,
//         }
//     }
// }
//
// impl<S: Iterator<Item=char>, K: Keypad<S>> Iterator for Simulate<S, K> {
//     type Item = Option<char>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let step = self.steps.next()?;
//         match step {
//             '^' => self.position.1 -= 1,
//             '>' => self.position.0 += 1,
//             'v' => self.position.1 += 1,
//             '<' => self.position.0 -= 1,
//             'A' => return Some(Some(K::coordinate_to_input(self.position))),
//             _ => panic!("Invalid step {step}"),
//         }
//         Some(None)
//     }
// }

#[derive(Debug, Clone)]
pub struct Combination {
    len: usize,
    parts: Rc<Vec<Route>>,
    current: usize,
    variants: usize,
    current_variance: usize,
}

impl PartialEq for Combination {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for Combination {}

impl Hash for Combination {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for route in self.iter() {
            route.hash(state);
        }
    }
}

impl Combination {
    pub fn new(parts: Rc<Vec<Route>>, variants: usize) -> Self {
        let len = parts.iter().map(|route| route.len()).sum();
        Self {
            len,
            parts,
            current: 0,
            variants,
            current_variance: 1,
        }
    }

    pub fn iter(&self) -> CombinationIter<'_> {
        CombinationIter {
            combination: self,
            current: 0,
            variants: self.variants,
            current_variance: 0,
        }
    }
}

pub struct CombinationIter<'combination> {
    combination: &'combination Combination,
    current: usize,
    variants: usize,
    current_variance: usize,
}

impl<'combination> ExactSizeIterator for CombinationIter<'combination> {
    fn len(&self) -> usize {
        self.combination.len
    }
}

impl<'combination> Iterator for CombinationIter<'combination> {
    type Item = Route;

    fn next(&mut self) -> Option<Self::Item> {
        let route = self.combination.parts.get(self.current)?;
        self.current += 1;
        if route.reversible() {
            if (&self.variants & &self.current_variance) != 0 {
                return Some(route.reverse());
            }
            self.current_variance <<= 1;
        }
        Some(route.clone())
    }
}

impl ExactSizeIterator for Combination {
    fn len(&self) -> usize {
        self.len
    }
}

impl Iterator for Combination {
    type Item = Route;

    fn next(&mut self) -> Option<Self::Item> {
        let route = self.parts.get(self.current)?;
        self.current += 1;
        if route.reversible() {
            if (&self.variants & &self.current_variance) != 0 {
                return Some(route.reverse());
            }
            self.current_variance <<= 1;
        }
        Some((*route).clone())
    }
}

#[derive(Debug, Clone)]
pub struct Combinations {
    parts: Rc<Vec<Route>>,
    current: usize,
    max: usize,
}

impl Hash for Combinations {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parts.hash(state);
    }
}

impl PartialEq<Self> for Combinations {
    fn eq(&self, other: &Self) -> bool {
        self.parts == other.parts
    }
}

impl Eq for Combinations { }


impl Combinations {
    pub fn new(parts: Vec<Route>) -> Self {
        let combinations = parts.iter()
            .filter(|route| route.reversible())
            .count();
        println!("{} {combinations}", parts.len());
        Self {
            parts: Rc::new(parts),
            current: 0,
            max: if combinations > 0 { 1 << (combinations - 1) } else { 0 },
        }
    }
}

impl Iterator for Combinations {
    type Item = Combination;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.max {
            return None;
        }

        let variants = self.current.clone();
        self.current += 1;

        Some(Combination::new(self.parts.clone(), variants))
    }
}

pub struct KeypadInput {
    cache: HashSet<Combinations>,
    combinations: Combinations,
}

impl KeypadInput {
    pub fn new(combinations: Combinations) -> Self {
        Self {
            cache: HashSet::new(),
            combinations,
        }
    }
}

impl Iterator for KeypadInput {
    type Item = Combinations;

    fn next(&mut self) -> Option<Self::Item> {
        let mut combination = Combinations::new(DirectionalKeypad::new(self.combinations.next()?).collect());
        while self.cache.contains(&combination) {
            combination = Combinations::new(DirectionalKeypad::new(self.combinations.next()?).collect());
        }
        Some(combination)
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

            let parts = NumericKeypad::new(code.chars()).collect::<Vec<_>>();
            // println!("{code} {parts:?}");
            // let mut combinations: Box<dyn Iterator<Item = Combinations>> = Box::new(std::iter::once(Combinations::new(parts)));
            let result = (0..2)
                .fold(
                    Box::new(std::iter::once(Combinations::new(parts))) as Box<dyn Iterator<Item = Combinations>>,
                    |state, _| {
                        Box::new(state.map(KeypadInput::new).flat_map(identity))
                    }
                )
                .flat_map(identity)
                .min_by(|a, b| a.len.cmp(&b.len))
                .unwrap();
            // println!("{code} {result:?} {}", result.len());
            // println!("{code} {solution:?} {}", solution.len());
            // println!("{code}: {} {}", current.first().unwrap().chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>(), current.first().unwrap().len());
            // println!("{code}: {} {}", solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>(), solution.len());
            // let simulated = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(result.chars()).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity).collect::<String>();
            // println!("{code}: simulated once {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated once {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // let simulated = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(result.chars()).filter_map(identity)).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity)).filter_map(identity).collect::<String>();
            // println!("{code}: simulated twice {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated twice {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // let simulated = Simulate::<_, NumericKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(result.chars()).filter_map(identity)).filter_map(identity)).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, NumericKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity)).filter_map(identity)).filter_map(identity).collect::<String>();
            // println!("{code}: simulated thrice {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated thrice {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            code_num * result.len()
        })
        .sum();

    Ok(result)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
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

            let parts = NumericKeypad::new(code.chars()).collect::<Vec<_>>();
            // println!("{code} {parts:?}");
            // let mut combinations: Box<dyn Iterator<Item = Combinations>> = Box::new(std::iter::once(Combinations::new(parts)));
            let result = (0..25)
                .fold(
                    Box::new(std::iter::once(Combinations::new(parts))) as Box<dyn Iterator<Item = Combinations>>,
                    |state, _| {
                        Box::new(state.map(KeypadInput::new).flat_map(identity))
                    }
                )
                .flat_map(identity)
                .min_by(|a, b| a.len.cmp(&b.len))
                .unwrap();
            // println!("{code} {result:?} {}", result.len());
            // println!("{code} {solution:?} {}", solution.len());
            // println!("{code}: {} {}", current.first().unwrap().chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>(), current.first().unwrap().len());
            // println!("{code}: {} {}", solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>(), solution.len());
            // let simulated = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(result.chars()).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity).collect::<String>();
            // println!("{code}: simulated once {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated once {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // let simulated = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(result.chars()).filter_map(identity)).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity)).filter_map(identity).collect::<String>();
            // println!("{code}: simulated twice {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated twice {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // let simulated = Simulate::<_, NumericKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(result.chars()).filter_map(identity)).filter_map(identity)).filter_map(identity).collect::<String>();
            // let simulated_solution = Simulate::<_, NumericKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(Simulate::<_, DirectionalKeypad<Chars<'_>>>::new(solution.chars()).filter_map(identity)).filter_map(identity)).filter_map(identity).collect::<String>();
            // println!("{code}: simulated thrice {}", simulated.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            // println!("{code}: simulated thrice {}", simulated_solution.chars().map(|symbol| if symbol == 'A' { symbol.bright_white().bold().to_string() } else { symbol.to_string() }).collect::<String>());
            code_num * result.len()
        })
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

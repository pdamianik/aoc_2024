use std::convert::identity;
use std::rc::Rc;
use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
// use itertools::Itertools;
// use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Coordinate, Direction, Grid};

pub const DAY: Day = Day(20);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    grid: Grid,
    start: usize,
    end: usize,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid: Grid = s.parse()?;

        let start = grid.as_slice().iter().position(|symbol| *symbol == 'S')
            .ok_or(eyre!("Failed to find start"))?;
        let end = grid.as_slice().iter().position(|symbol| *symbol == 'E')
            .ok_or(eyre!("Failed to find end"))?;

        Ok(Self {
            grid,
            start,
            end,
        })
    }
}

pub fn process_part1<const SAVE: usize>(input: &Input) -> eyre::Result<usize> {
    let distances = input.grid.flood(input.start, |tile| tile == '#');
    // let max_distance = distances.iter()
    //     .filter(|&&distance| distance != usize::MAX)
    //     .map(|distance| if *distance == 0 { 1 } else { distance.ilog10() + 1 })
    //     .max()
    //     .unwrap();

    let result = input.grid.as_slice().iter()
        .enumerate()
        .filter(|&(_, &tile)| tile == '.' || tile == 'S')
        .map(|(anchor, _)| {
            Direction::ALL
                .iter()
                .filter_map(|direction|
                    input.grid.offset_index(anchor, (*direction).into()).ok()
                        .map(|position| (direction, position))
                )
                .flat_map(|(direction, position)| {
                    [direction.clone(), direction.rotate90()]
                        .into_iter()
                        .filter_map(move |direction|
                            input.grid.offset_index(position, direction.into()).ok()
                                .map(|position| position)
                        )
                })
                // .inspect(|position| {
                //     if anchor == input.start {
                //         let max_row = distances.len().ilog10();
                //         println!("from {anchor} to {position}");
                //         println!("{}", distances.as_slice().iter().zip(input.grid.as_slice()).enumerate().chunks(input.grid.width()).into_iter().enumerate().map(|(row, distances)| {
                //             format!("{}{}{}", " ".repeat((max_row - if row == 0 { 0 } else { (row * input.grid.width()).ilog10() }) as usize), row * input.grid.width(), distances.map(|(tile_position, (distance, tile))| {
                //                 let cell = if *distance == usize::MAX {
                //                     format!("{}{}", " ".repeat(max_distance as usize), tile)
                //                 } else if *distance == 0 {
                //                     format!("{}{}", " ".repeat(max_distance as usize), 0)
                //                 } else {
                //                     format!("{}{}", " ".repeat((max_distance - distance.ilog10()) as usize), distance)
                //                 };
                //                 if tile_position == anchor {
                //                     cell.bright_green().bold().to_string()
                //                 } else if tile_position == *position {
                //                     cell.bright_red().bold().to_string()
                //                 } else if *tile == '#' {
                //                     cell.white().to_string()
                //                 } else {
                //                     cell.bright_white().bold().to_string()
                //                 }
                //             }).collect::<String>())
                //         }).join("\n"));
                //     }
                // })
                .filter(|&position| input.grid.as_slice()[position] != '#')
                .filter(|&position| distances[anchor] + SAVE + 2 <= distances[position])
                .count()
        })
        .sum();

    Ok(result)
}

pub struct Offsets<'grid, const MAX: usize> {
    anchor: usize,
    straight: Coordinate,
    queer: Coordinate,
    straight_offset: usize,
    queer_offset: usize,
    grid: &'grid Grid,
    finished: bool,
}

impl<'grid, const MAX: usize> Offsets<'grid, MAX> {
    pub fn new(start: usize, direction: Direction, grid: &'grid Grid) -> Self {
        Self {
            anchor: start,
            straight: direction.into(),
            queer: direction.rotate90().into(),
            straight_offset: MAX,
            queer_offset: 0,
            grid,
            finished: false,
        }
    }
}

impl<const MAX: usize> Iterator for Offsets<'_, MAX> {
    type Item = Option<(usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let offset = self.straight * self.straight_offset as isize + self.queer * self.queer_offset as isize;
        let distance = self.straight_offset + self.queer_offset;
        let position = self.grid.offset_index(self.anchor, offset);

        if self.queer_offset == 0 {
            if self.straight_offset == 0 {
                self.finished = true;
            } else {
                self.straight_offset -= 1;
                self.queer_offset = MAX - self.straight_offset;
            }
        } else {
            self.queer_offset -= 1;
        }

        Some(position.ok().map(|position| (position, distance)))
    }
}

pub fn process_part2<const SAVE: usize>(input: &Input) -> eyre::Result<usize> {
    let distances = Rc::new(input.grid.flood(input.start, |tile| tile == '#'));
    // let max_distance = distances.iter()
    //     .filter(|&&distance| distance != usize::MAX)
    //     .map(|distance| if *distance == 0 { 1 } else { distance.ilog10() + 1 })
    //     .max()
    //     .unwrap();

    // println!("{}", distances.as_slice().iter().zip(input.grid.as_slice()).chunks(input.grid.width()).into_iter().map(|distances| {
    //     distances.map(|(distance, tile)| {
    //         if *distance == usize::MAX {
    //             format!("{}{}", " ".repeat(max_distance as usize), tile.white().to_string())
    //         } else if *distance == 0 {
    //             format!("{}{}", " ".repeat(max_distance as usize), 0)
    //         } else {
    //             format!("{}{}", " ".repeat((max_distance - distance.ilog10()) as usize), distance)
    //         }
    //     }).collect::<String>()
    // }).join("\n"));

    let skips = input.grid.as_slice().iter()
        .enumerate()
        .filter(|&(_, &tile)| tile == '.' || tile == 'S')
        .flat_map(|(anchor, _)| {
            let distances = distances.clone();
            Direction::ALL
                .iter()
                .filter_map(move |direction|
                    input.grid.offset_index(anchor, (*direction).into()).ok()
                        .map(|position| (direction, position))
                )
                .flat_map(|(direction, position)| Offsets::<19>::new(position, *direction, &input.grid))
                .filter_map(identity)
                .filter(|&(position, _)| input.grid.as_slice()[position] != '#')
                .map(|(position, distance)| (position, distance + 1))
                .filter_map(move |(position, distance)| if distances[anchor] + distance < distances[position] {
                    Some((anchor, position, distances[position] - distances[anchor] - distance))
                } else {
                    None
                })
                .filter(move |&(_, _, saved)| saved >= SAVE)
        })
        .collect::<Vec<_>>();

    // for &(from, to, saved) in &skips {
    //     // if anchor == input.start {
    //         let max_row = distances.len().ilog10();
    //         println!("from {from} to {to} distance {} saving {}", distances[to] as isize - distances[from] as isize, saved);
    //         println!("{}", distances.as_slice().iter().zip(input.grid.as_slice()).enumerate().chunks(input.grid.width()).into_iter().enumerate().map(|(row, distances)| {
    //             format!("{}{}{}", " ".repeat((max_row - if row == 0 { 0 } else { (row * input.grid.width()).ilog10() }) as usize), row * input.grid.width(), distances.map(|(tile_position, (distance, tile))| {
    //                 let cell = if *distance == usize::MAX {
    //                     format!("{}{}", " ".repeat(max_distance as usize), tile)
    //                 } else if *distance == 0 {
    //                     format!("{}{}", " ".repeat(max_distance as usize), 0)
    //                 } else {
    //                     format!("{}{}", " ".repeat((max_distance - distance.ilog10()) as usize), distance)
    //                 };
    //                 if tile_position == from {
    //                     cell.bright_green().bold().to_string()
    //                 } else if tile_position == to {
    //                     cell.bright_red().bold().to_string()
    //                 } else if *tile == '#' {
    //                     cell.white().to_string()
    //                 } else {
    //                     cell.bright_white().bold().to_string()
    //                 }
    //             }).collect::<String>())
    //         }).join("\n"));
    //     // }
    // }

    let result = skips.len();

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
        let result1 = process_part1::<100>(&input)?;
        let end1 = SystemTime::now();
        let start2 = SystemTime::now();
        let result2 = process_part2::<100>(&input)?;
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
        r"###############
          #...#...#.....#
          #.#.#.#.#.###.#
          #S#...#.#.#...#
          #######.#.#.###
          #######.#.#...#
          #######.#.###.#
          ###..E#...#...#
          ###.#######.###
          #...###...#...#
          #.#####.#.###.#
          #.#...#.#.#...#
          #.#.#.#.#.#.###
          #...#...#...###
          ###############
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1::<2>(&input).unwrap();
        assert_eq!(14 + 14 + 2 + 4 + 2 + 3 + 5, result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2::<50>(&input).unwrap();
        assert_eq!(32 + 31 + 29 + 39 + 25 + 23 + 20 + 19 + 12 + 14 + 12 + 22 + 4 + 3, result);
    }
}

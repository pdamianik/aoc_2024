use std::collections::VecDeque;
use std::str::FromStr;
use std::time::SystemTime;
use itertools::Itertools;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Coordinate, Grid};

pub const DAY: Day = Day(12);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    grid: Grid,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s.parse()?;

        Ok(Self {
            grid,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let mut visited = vec![false; input.grid.as_slice().len()];
    let mut search_positions = VecDeque::new();
    let mut cost = 0;

    for (position, plot) in input.grid.as_slice().iter().enumerate() {
        if visited[position] {
            continue;
        }
        // for row in visited.chunks(input.grid.width()) {
        //     println!("{}", row.iter().map(|visited| if *visited { 'X' } else { '0' }).collect::<String>());
        // }

        let mut area = 0;
        let mut perimeter = 0;
        search_positions.push_back(position);
        visited[position] = true;
        while let Some(search_position) = search_positions.pop_front() {
            // println!("adding {search_position} to area");
            area += 1;

            for direction in Coordinate::CARDINALITIES {
                // print!("searching {search_position} in {direction:?}: ");
                if let Ok(new_position) = input.grid.offset_index(search_position, direction) {
                    if visited[new_position] || input.grid.as_slice()[new_position] != *plot {
                        if input.grid.as_slice()[new_position] != *plot {
                            perimeter += 1;
                            // println!("different plant type");
                        } else {
                            // println!("already visited");
                        }
                        continue;
                    }

                    visited[new_position] = true;
                    search_positions.push_back(new_position);
                    // println!("new plot");
                } else {
                    perimeter += 1;
                    // println!("outside");
                }
            }
        }
        // println!("area: {area}, perimeter: {perimeter}");
        cost += area * perimeter;
    }

    Ok(cost)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let mut visited = vec![false; input.grid.as_slice().len()];
    let mut search_positions = VecDeque::new();
    let mut cost = 0;

    for (position, plot) in input.grid.as_slice().iter().enumerate() {
        if visited[position] {
            continue;
        }

        let mut area = 0;
        let mut perimeter = 0;
        search_positions.push_back(position);
        while let Some(search_position) = search_positions.pop_front() {
            if visited[search_position] {
                continue;
            }
            area += 1;
            visited[search_position] = true;

            let grid = input.grid.as_slice();
            for (&direction1, &direction2) in Coordinate::CARDINALITIES.iter().chain(std::iter::once(&Coordinate::CARDINALITIES[0])).tuple_windows() {
                let direction1_inside = input.grid.offset_index(search_position, direction1).is_ok_and(|position| grid[position] == *plot);
                let direction2_inside = input.grid.offset_index(search_position, direction2).is_ok_and(|position| grid[position] == *plot);
                let direction3_inside = input.grid.offset_index(search_position, direction1 + direction2).is_ok_and(|position| grid[position] == *plot);

                if !direction1_inside && !direction2_inside {
                    perimeter += 1;
                } else if direction1_inside && direction2_inside && !direction3_inside {
                    perimeter += 1;
                } else {
                }

                if let Ok(direction1) = input.grid.offset_index(search_position, direction1) {
                    if input.grid.as_slice()[direction1] == *plot {
                        search_positions.push_back(direction1);
                    }
                }
            }
        }
        cost += area * perimeter;
    }

    Ok(cost)
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
    pub fn test_example1_part1() {
        let input = r"AAAA
                           BBCD
                           BBCC
                           EEEC
                           ".parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!(140, result);
    }

    #[test]
    pub fn test_example2_part1() {
        let input = r"OOOOO
                            OXOXO
                            OOOOO
                            OXOXO
                            OOOOO
                            ".parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!(772, result);
    }

    #[test]
    pub fn test_example3_part1() {
        let input = r"RRRRIICCFF
                            RRRRIICCCF
                            VVRRRCCFFF
                            VVRCCCJFFF
                            VVVVCJJCFE
                            VVIVCCJJEE
                            VVIIICJJEE
                            MIIIIIJJEE
                            MIIISIJEEE
                            MMMISSJEEE
                            ".parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!(1930, result);
    }

    #[test]
    pub fn test_example1_part2() {
        let input = r"AAAA
                           BBCD
                           BBCC
                           EEEC
                           ".parse().unwrap();

        let result = process_part2(&input).unwrap();
        assert_eq!(80, result);
    }

    #[test]
    pub fn test_example2_part2() {
        let input = r"EEEEE
                            EXXXX
                            EEEEE
                            EXXXX
                            EEEEE
                            ".parse().unwrap();

        let result = process_part2(&input).unwrap();
        assert_eq!(236, result);
    }

    #[test]
    pub fn test_example3_part2() {
        let input = r"AAAAAA
                            AAABBA
                            AAABBA
                            ABBAAA
                            ABBAAA
                            AAAAAA
                            ".parse().unwrap();

        let result = process_part2(&input).unwrap();
        assert_eq!(368, result);
    }

    #[test]
    pub fn test_example4_part1() {
        let input = r"RRRRIICCFF
                            RRRRIICCCF
                            VVRRRCCFFF
                            VVRCCCJFFF
                            VVVVCJJCFE
                            VVIVCCJJEE
                            VVIIICJJEE
                            MIIIIIJJEE
                            MIIISIJEEE
                            MMMISSJEEE
                            ".parse().unwrap();

        let result = process_part2(&input).unwrap();
        assert_eq!(1206, result);
    }
}

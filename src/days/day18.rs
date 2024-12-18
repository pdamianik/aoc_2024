use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Coordinate, Direction, ParsedGrid};

pub const DAY: Day = Day(18);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    byte_locations: Vec<(usize, usize)>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let byte_locations = s.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.split_once(",").unwrap())
            .map(|(x, y)| (x.parse().unwrap(), y.parse().unwrap()))
            .collect();

        Ok(Self {
            byte_locations,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Node {
    position: usize,
    distance: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn process_part1<const WIDTH: usize, const HEIGHT: usize, const INITIAL: usize>(input: &Input) -> eyre::Result<usize> {
    let mut grid = ParsedGrid::new(WIDTH, HEIGHT);

    for (x, y) in &input.byte_locations[0..INITIAL] {
        grid.as_mut_slice()[y * WIDTH + x] = true;
    }

    let mut to_visit = BinaryHeap::new();
    let mut distances = vec![usize::MAX; grid.as_slice().len()];

    distances[0] = 0;
    to_visit.push(Node { position: 0, distance: 0 });

    while let Some(Node { position, distance }) = to_visit.pop() {
        for direction in Direction::ALL {
            let position = if let Ok(position) = grid.offset_index(position, direction.into()) {
                position
            } else {
                continue;
            };
            let distance = distance + 1;

            if distance < distances[position] && !grid.as_slice()[position] {
                distances[position] = distance;
                to_visit.push(Node { position, distance });
            }
        }
    }

    // let max_length = distances.iter().filter(|distance| **distance != usize::MAX).max().unwrap().ilog10() + 1;
    // println!("{}", distances.iter()
    //     .zip(grid.as_slice())
    //     .map(|(&distance, &corrupted)| {
    //         let cell_length = if corrupted || distance == usize::MAX {
    //             1
    //         } else if distance == 0 {
    //             1
    //         } else {
    //             distance.ilog10() + 1
    //         };
    //         let cell = if corrupted {
    //             "#".white().to_string()
    //         } else if distance == usize::MAX {
    //             "#".bold().bright_red().to_string()
    //         } else {
    //             distance.to_string()
    //         };
    //         format!("{}{cell}", " ".repeat((max_length - cell_length) as usize))
    //     })
    //     .chunks(WIDTH)
    //     .into_iter()
    //     .map(|mut line| line.join(" "))
    //     .join("\n")
    // );

    Ok(distances[WIDTH * HEIGHT - 1])
}

pub fn process_part2<const WIDTH: usize, const HEIGHT: usize, const INITIAL: usize>(input: &Input) -> eyre::Result<Coordinate> {
    let mut grid = ParsedGrid::new(WIDTH, HEIGHT);

    for (x, y) in &input.byte_locations[0..INITIAL] {
        grid.as_mut_slice()[y * WIDTH + x] = true;
    }

    let mut to_visit = VecDeque::new();
    let mut distances = vec![usize::MAX; grid.as_slice().len()];

    distances[0] = 0;
    to_visit.push_back(Node { position: 0, distance: 0 });

    while let Some(Node { position, distance }) = to_visit.pop_front() {
        for direction in Direction::ALL {
            let position = if let Ok(position) = grid.offset_index(position, direction.into()) {
                position
            } else {
                continue;
            };
            let distance = distance + 1;

            if distance < distances[position] && !grid.as_slice()[position] {
                distances[position] = distance;
                to_visit.push_back(Node { position, distance });
            }
        }
    }

    // let max_length = distances.iter().filter(|distance| **distance != usize::MAX).max().unwrap().ilog10() + 1;
    // println!("{}", distances.iter()
    //     .zip(grid.as_slice())
    //     .map(|(&distance, &corrupted)| {
    //         let cell_length = if corrupted || distance == usize::MAX || distance == 0 {
    //             1
    //         } else {
    //             distance.ilog10() + 1
    //         };
    //         let cell = if corrupted {
    //             "#".white().to_string()
    //         } else if distance == usize::MAX {
    //             "#".bold().bright_red().to_string()
    //         } else {
    //             distance.to_string()
    //         };
    //         format!("{}{cell}", " ".repeat((max_length - cell_length) as usize))
    //     })
    //     .chunks(WIDTH)
    //     .into_iter()
    //     .map(|mut line| line.join(" "))
    //     .join("\n")
    // );

    for (x, y) in &input.byte_locations[INITIAL..] {
        let coordinate = Coordinate(*x as isize, *y as isize);
        // println!("Corrupting {coordinate}");
        let corruption_position = *y * WIDTH + *x;
        grid.as_mut_slice()[corruption_position] = true;
        let blocked_distance = distances[corruption_position];
        let rescan = distances.iter_mut()
            .enumerate()
            .filter_map(|(position, distance)| {
                if grid.as_slice()[position] {
                    None
                } else if *distance > blocked_distance {
                    *distance = usize::MAX;
                    None
                } else if blocked_distance != usize::MAX && *distance == blocked_distance {
                    Some((position, *distance))
                } else {
                    None
                }
            })
            .map(|(position, distance)| Node { position, distance });
        to_visit.extend(rescan);

        // let max_length = distances.iter().filter(|distance| **distance != usize::MAX).max().unwrap().ilog10() + 1;
        // println!("{}\n", distances.iter()
        //     .zip(grid.as_slice())
        //     .enumerate()
        //     .map(|(position, (&distance, &corrupted))| {
        //         let cell_length = if corrupted || distance == usize::MAX || distance == 0 {
        //             1
        //         } else {
        //             distance.ilog10() + 1
        //         };
        //         let cell = if position == corruption_position {
        //             "#".bold().bright_yellow().to_string()
        //         } else if corrupted {
        //             "#".white().to_string()
        //         } else if distance == usize::MAX {
        //             "#".bold().bright_red().to_string()
        //         } else {
        //             distance.to_string()
        //         };
        //         format!("{}{cell}", " ".repeat((max_length - cell_length) as usize))
        //     })
        //     .chunks(WIDTH)
        //     .into_iter()
        //     .map(|mut line| line.join(" "))
        //     .join("\n")
        // );

        while let Some(Node { position, distance }) = to_visit.pop_front() {
            for direction in Direction::ALL {
                let position = if let Ok(position) = grid.offset_index(position, direction.into()) {
                    position
                } else {
                    continue;
                };
                let distance = distance + 1;

                if distance < distances[position] && !grid.as_slice()[position] {
                    distances[position] = distance;
                    to_visit.push_back(Node { position, distance });
                }
            }
        }

        // let max_length = distances.iter().filter(|distance| **distance != usize::MAX).max().unwrap().ilog10() + 1;
        // println!("{}", distances.iter()
        //     .zip(grid.as_slice())
        //     .map(|(&distance, &corrupted)| {
        //         let cell_length = if corrupted || distance == usize::MAX || distance == 0 {
        //             1
        //         } else {
        //             distance.ilog10() + 1
        //         };
        //         let cell = if corrupted {
        //             "#".white().to_string()
        //         } else if distance == usize::MAX {
        //             "#".bold().bright_red().to_string()
        //         } else {
        //             distance.to_string()
        //         };
        //         format!("{}{cell}", " ".repeat((max_length - cell_length) as usize))
        //     })
        //     .chunks(WIDTH)
        //     .into_iter()
        //     .map(|mut line| line.join(" "))
        //     .join("\n")
        // );

        if distances[WIDTH * HEIGHT - 1] == usize::MAX {
            return Ok(coordinate);
        }
    }

    Err(eyre!("Could not find any corruption that blocks the path"))
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
        let result1 = process_part1::<71, 71, 1024>(&input)?;
        let end1 = SystemTime::now();
        let start2 = SystemTime::now();
        let result2 = process_part2::<71, 71, 1024>(&input)?;
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
    pub fn test_example_part1() {
        let input = r"5,4
                            4,2
                            4,5
                            3,0
                            2,1
                            6,3
                            2,4
                            1,5
                            0,6
                            3,3
                            2,6
                            5,1
                            1,2
                            5,5
                            2,5
                            6,5
                            1,4
                            0,4
                            6,4
                            1,1
                            6,1
                            1,0
                            0,5
                            1,6
                            2,0
                            ".parse().unwrap();

        let result = process_part1::<7, 7, 12>(&input).unwrap();
        assert_eq!(22, result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = r"5,4
                            4,2
                            4,5
                            3,0
                            2,1
                            6,3
                            2,4
                            1,5
                            0,6
                            3,3
                            2,6
                            5,1
                            1,2
                            5,5
                            2,5
                            6,5
                            1,4
                            0,4
                            6,4
                            1,1
                            6,1
                            1,0
                            0,5
                            1,6
                            2,0
                            ".parse().unwrap();

        let result = process_part2::<7, 7, 12>(&input).unwrap();
        assert_eq!(Coordinate(6, 1), result);
    }
}

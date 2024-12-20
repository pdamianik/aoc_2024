use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::str::FromStr;
use std::time::SystemTime;
// use anes::{ClearBuffer, HideCursor, MoveCursorTo, ShowCursor};
use eyre::eyre;
// use itertools::Itertools;
// use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Direction, Grid};

pub const DAY: Day = Day(16);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct State {
    pub position: usize,
    pub score: usize,
    pub facing: Direction,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.cmp(&self.score)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct Input {
    map: Grid,
    start: usize,
    end: usize,
}

impl Input {
    pub fn find_score(&self) -> Option<usize> {
        let mut scores = vec![usize::MAX; self.map.as_slice().len()];
        let mut heap = BinaryHeap::new();

        scores[self.start] = 0;
        heap.push(State { position: self.start, score: 0, facing: Direction::East });

        while let Some(State { position, score, facing }) = heap.pop() {
            if position == self.end {
                return Some(score);
            }

            if score > scores[position] { continue; }

            for direction in Direction::ALL {
                if direction == facing.rotate180() {
                    continue;
                }
                let score = if direction == facing {
                    score + 1
                } else {
                    score + 1001
                };
                let position = if let Ok(position) = self.map.offset_index(position, direction.into()) {
                    position
                } else {
                    continue;
                };
                if self.map.as_slice()[position] == '#' {
                    continue;
                }
                let next = State { score, position, facing: direction };

                if next.score < scores[next.position] {
                    scores[next.position] = next.score;
                    heap.push(next);
                }
            }
        }

        None
    }

    pub fn count_best_paths(&self) -> usize {
        let mut scores = vec![(usize::MAX, 0u8); self.map.as_slice().len()];
        let mut heap = BinaryHeap::new();

        scores[self.start].0 = 0;
        heap.push(State { position: self.start, score: 0, facing: Direction::East });

        // let mut count = 0;
        // print!("{}{}", ClearBuffer::All, HideCursor);
        while let Some(State { position, score, facing }) = heap.pop() {
            // if (count & ((1 << 4) - 1)) == 0 {
            //     println!("{}{}", MoveCursorTo(0, 0), scores.iter()
            //         .map(|&(_, directions)| if directions != 0 {
            //             display_directions(directions).dimmed().bright_white().to_string()
            //         } else {
            //             display_directions(directions).dimmed().white().to_string()
            //         })
            //         .chunks(self.map.width()).into_iter()
            //         .map(|chunk| chunk.collect::<String>())
            //         .join("\n")
            //     );
            // }
            if score > scores[self.end].0 {
                break;
            }

            if score > scores[position].0 { continue; }

            let previous_direction = facing.rotate180();
            for direction in Direction::ALL {
                if direction == previous_direction {
                    continue;
                }
                let score = if direction == facing {
                    score + 1
                } else {
                    score + 1001
                };
                let position = if let Ok(position) = self.map.offset_index(position, direction.into()) {
                    position
                } else {
                    continue;
                };
                if self.map.as_slice()[position] == '#' {
                    continue;
                }
                let next = State { score, position, facing: direction };

                scores[position].1 |= direction.rotate180().mask();
                if next.score < scores[next.position].0 {
                    scores[next.position].0 = next.score;
                    heap.push(next);
                }
            }
            // count += 1;
        }
        let mut shortest_path = VecDeque::from([(self.end, None)]);
        let mut shortest_map = vec![false; self.map.as_slice().len()];
        // count = 0;
        while let Some((shortest_element, previous)) = shortest_path.pop_front() {
            shortest_map[shortest_element] = true;
            let mask = scores[shortest_element].1;
            for direction in Direction::from_mask(mask) {
                let position = self.map.offset_index(shortest_element, direction.into()).unwrap();
                let tolerance = if previous.is_some_and(|previous| direction == previous) {
                    1000usize
                } else {
                    0
                };
                if scores[position].0  < scores[shortest_element].0 + tolerance {
                    shortest_path.push_back((position, Some(direction)));
                }
            }
            // if (count & ((1 << 6) - 1)) == 0 {
            //     println!("{}{}", MoveCursorTo(0, 0), scores.iter()
            //         .zip(shortest_map.iter())
            //         .map(|(&(_, directions), &shortest_route)| if shortest_route {
            //             display_directions(directions).bold().bright_green().to_string()
            //         } else if directions != 0 {
            //             display_directions(directions).dimmed().bright_white().to_string()
            //         } else {
            //             display_directions(directions).dimmed().white().to_string()
            //         })
            //         .chunks(self.map.width()).into_iter()
            //         .map(|chunk| chunk.collect::<String>())
            //         .join("\n")
            //     );
            // }
            // count += 1;
        }
        // println!("{}{}", MoveCursorTo(0, 0), scores.iter()
        //     .zip(shortest_map.iter())
        //     .map(|(&(_, directions), &shortest_route)| if shortest_route {
        //         display_directions(directions).bold().bright_green().to_string()
        //     } else if directions != 0 {
        //         display_directions(directions).dimmed().bright_white().to_string()
        //     } else {
        //         display_directions(directions).dimmed().white().to_string()
        //     })
        //     .chunks(self.map.width()).into_iter()
        //     .map(|chunk| chunk.collect::<String>())
        //     .join("\n")
        // );
        // print!("{}", ShowCursor);
        shortest_map.iter().filter(|shortest| **shortest).count()
    }
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = s.parse::<Grid>()?;
        let start = map.as_slice().iter()
            .position(|character| *character == 'S')
            .ok_or(eyre!("Failed to find 'S' marking the start position"))?;
        let end = map.as_slice().iter()
            .position(|character| *character == 'E')
            .ok_or(eyre!("Failed to find 'E' marking the start position"))?;

        Ok(Self {
            map,
            start,
            end,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let result = input.find_score().unwrap();

    Ok(result)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let result = input.count_best_paths();

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

    fn example_1_input() -> Input {
        r"###############
          #.......#....E#
          #.#.###.#.###.#
          #.....#.#...#.#
          #.###.#####.#.#
          #.#.#.......#.#
          #.#.#####.###.#
          #...........#.#
          ###.#.#####.#.#
          #...#.....#.#.#
          #.#.#.###.#.#.#
          #.....#...#.#.#
          #.###.#.#.#.#.#
          #S..#.....#...#
          ###############
          ".parse().unwrap()
    }

    fn example_2_input() -> Input {
        r"#################
          #...#...#...#..E#
          #.#.#.#.#.#.#.#.#
          #.#.#.#...#...#.#
          #.#.#.#.###.#.#.#
          #...#.#.#.....#.#
          #.#.#.#.#.#####.#
          #.#...#.#.#.....#
          #.#.#####.#.###.#
          #.#.#.......#...#
          #.#.###.#####.###
          #.#.#...#.....#.#
          #.#.#.#####.###.#
          #.#.#.........#.#
          #.#.#.#########.#
          #S#.............#
          #################
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_1_part1() {
        let input = example_1_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(7036, result);
    }

    #[test]
    pub fn test_example_2_part1() {
        let input = example_2_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(11048, result);
    }

    #[test]
    pub fn test_example_1_part2() {
        let input = example_1_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(45, result);
    }

    #[test]
    pub fn test_example_2_part2() {
        let input = example_2_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(64, result);
    }

    // from https://www.reddit.com/r/adventofcode/comments/1hfhgl1/2024_day_16_part_1_alternate_test_case/
    fn alternate_input() -> Input {
        include_str!("../../test/input/day16_alternate.in").parse().unwrap()
    }

    #[test]
    pub fn test_alternate_part1() {
        let input = alternate_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(21148, result);
    }

    #[test]
    pub fn test_alternate_part2() {
        let input = alternate_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(149, result);
    }
}

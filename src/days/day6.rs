use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::time::SystemTime;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(6);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rotation(u8);

impl Deref for Rotation {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Rotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            Self::NORTH => 'N',
            Self::EAST => 'E',
            Self::SOUTH => 'S',
            Self::WEST => 'W',
            _ => unreachable!(),
        })
    }
}

impl Rotation {
    pub const ALL: [Self; 4] = [Self::NORTH, Self::EAST, Self::SOUTH, Self::WEST];
    pub const NORTH: Self = Self(1 << 0);
    pub const EAST: Self = Self(1 << 1);
    pub const SOUTH: Self = Self(1 << 2);
    pub const WEST: Self = Self(1 << 3);

    pub fn rotate90(mut self) -> Self {
        self.0 <<= 1;
        if self.0 == 1 << 4 {
            self.0 = 1;
        }
        Self(self.0)
    }

    pub fn rotate270(mut self) -> Self {
        if self.0 == 1 {
            self.0 = 1 << 4;
        }
        self.0 >>= 1;
        Self(self.0)
    }

    pub fn index(&self) -> usize {
        match *self {
            Self::NORTH => 0,
            Self::EAST => 1,
            Self::SOUTH => 2,
            Self::WEST => 3,
            _ => unreachable!(),
        }
    }

    pub fn go(&self, position: usize, width: usize) -> usize {
        match *self {
            Self::NORTH => position - width,
            Self::EAST => position + 1,
            Self::SOUTH => position + width,
            Self::WEST => position - 1,
            _ => unreachable!(),
        }
    }
}

fn display_directions(directions: u8) -> char {
    [
        '.', // 0b0000
        '╵', // 0b0001
        '╶', // 0b0010
        '└', // 0b0011
        '╷', // 0b0100
        '│', // 0b0101
        '┌', // 0b0110
        '├', // 0b0111
        '╴', // 0b1000
        '┘', // 0b1001
        '─', // 0b1010
        '┴', // 0b1011
        '┐', // 0b1100
        '┤', // 0b1101
        '┬', // 0b1110
        '┼', // 0b1111
    ][directions as usize]
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position {
    pub position: usize,
    pub direction: Rotation,
    pub width: usize,
    pub height: usize,
}

impl Position {
    pub fn look(&self) -> Option<usize> {
        match self.direction {
            Rotation::NORTH => {
                if self.position >= self.width {
                    Some(self.position - self.width)
                } else {
                    None
                }
            }
            Rotation::EAST => {
                if self.position % self.width < self.width - 1 {
                    Some(self.position + 1)
                } else {
                    None
                }
            }
            Rotation::SOUTH => {
                if self.position < (self.height - 1) * self.width {
                    Some(self.position + self.width)
                } else {
                    None
                }
            }
            Rotation::WEST => {
                if self.position % self.width > 0 {
                    Some(self.position - 1)
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn look_back(&self) -> Option<usize> {
        match self.direction {
            Rotation::NORTH => {
                if self.position < (self.height - 1) * self.width {
                    Some(self.position + self.width)
                } else {
                    None
                }
            }
            Rotation::EAST => {
                if self.position % self.width > 0 {
                    Some(self.position - 1)
                } else {
                    None
                }
            }
            Rotation::SOUTH => {
                if self.position >= self.width {
                    Some(self.position - self.width)
                } else {
                    None
                }
            }
            Rotation::WEST => {
                if self.position % self.width < self.width - 1 {
                    Some(self.position + 1)
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self) -> Option<usize> {
        self.position = self.look()?;
        Some(self.position)
    }

    pub fn rotate90(&mut self) {
        self.direction = self.direction.rotate90();
    }

    pub fn rotate270(&mut self) {
        self.direction = self.direction.rotate270();
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Input {
    char_map: Vec<char>,
    position: Position,
}

impl Input {
    pub fn step(&mut self) -> Option<usize> {
        let new_position = self.position.look()?;
        if self.char_map[new_position] == '#' {
            self.position.rotate90();
            Some(self.position.position)
        } else {
            self.position.step()
        }
    }
}

impl FromStr for Input {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .join("\n");
        let width = s.find('\n').unwrap();
        let char_map: Vec<char> = s.lines().map(|line| line.chars()).flatten().collect();

        let position = Position {
            position: char_map.iter().position(|character| *character == '^').unwrap(),
            direction: Rotation::NORTH,
            width,
            height: char_map.len() / width,
        };

        Ok(Self {
            char_map,
            position,
        })
    }
}

fn movement_map(input: &Input) -> Result<Vec<u8>, Vec<u8>> {
    let mut map = input.clone();
    let mut visited = vec![0u8; map.char_map.len()];
    visited[map.position.position] |= *map.position.direction;

    while let Some(position) = map.step() {
        // visiting a position in the same direction twice is a loop
        if visited[position] & *map.position.direction != 0 {
            return Err(visited);
        }
        visited[position] |= *map.position.direction;
    }

    Ok(visited)
}

pub fn process_part1(input: &Input) -> eyre::Result<String> {
    let visited = movement_map(input).unwrap();
    // println!("{}\n", visualize_visited(&visited, input.position.width));

    let result: usize = visited.into_iter().filter(|&directions| directions != 0).count();

    Ok(result.to_string())
}

#[allow(dead_code)]
fn visualize_visited(visited: &[(u8, [Option<usize>; 4])], width: usize) -> String {
    visited.chunks(width)
        .into_iter()
        .map(|row| row.iter().map(|&(visited, _)| if visited != 0 { '1'.bright_green().bold().to_string() } else { '0'.dimmed().to_string() }).join(""))
        .join("\n")
}

#[allow(dead_code)]
fn visualize_paths(input: &Input, visited: &[u8], width: usize, obstacle: Option<usize>, direction: Option<(usize, Rotation)>, mark: Option<usize>, new_obstacle: Option<usize>) -> String {
    visited.iter().enumerate().chunks(width)
        .into_iter()
        .map(|row|
            row.into_iter().map(|(position, &directions)| {
                let text = if input.char_map[position] == '#' {
                    '#'
                } else {
                    display_directions(directions)
                };
                match (position, obstacle, direction, mark, new_obstacle) {
                    (position, Some(obstacle), _, _, _) if obstacle == position => text.bold().bright_red().to_string(),
                    (position, _, Some(direction), _, _) if direction.0 == position => direction.1.bold().bright_yellow().to_string(),
                    (position, _, _, Some(mark), _) if mark == position => text.bold().bright_green().to_string(),
                    (position, _, _, _, Some(new_obstacle)) if position == new_obstacle => "O".bold().bright_blue().to_string(),
                    _ => text.dimmed().to_string(),
                }
            })
                .join("")
        )
        .join("\n")
}

#[allow(dead_code)]
fn visualize_visited_time(visited: &[(u8, [Option<usize>; 4])], width: usize, obstacle: Option<usize>, direction: Option<(usize, Rotation)>, mark: Option<usize>, new_obstacle: Option<usize>) -> String {
    let times = visited
        .into_iter()
        .map(|(_, times)| times.iter().filter_map(|time| *time).next().unwrap_or(0))
        .collect::<Vec<_>>();

    let max = *times.iter().max().unwrap();
    let max_len = max.ilog10() as usize;

    times.iter().enumerate().chunks(width)
        .into_iter()
        .map(|times|
            times.into_iter().map(|(position, time)| {
                let len = if *time == 0 {
                    0usize
                } else {
                    time.ilog10() as usize
                };
                let filler = " ".repeat((max_len - len) as usize);
                let text = format!("{}{}", filler, time);
                match (position, obstacle, direction, mark, new_obstacle) {
                    (position, Some(obstacle), _, _, _) if obstacle == position => text.bold().bright_red().to_string(),
                    (position, _, Some(direction), _, _) if direction.0 == position => format!("{}{}", " ".repeat(max_len), direction.1).bold().bright_yellow().to_string(),
                    (position, _, _, Some(mark), _) if mark == position => text.bold().bright_green().to_string(),
                    (position, _, _, _, Some(new_obstacle)) if position == new_obstacle => format!("{}O", " ".repeat(max_len)).bold().bright_blue().to_string(),
                    _ if *time == 0 => text.dimmed().to_string(),
                    _ => text.bold().to_string(),
                }
            })
                .join(" ")
        )
        .join("\n")
}

// fn search_obstacle(start: &usize, obstacle: usize, rotation: Rotation, movement_map: &[u8], input: &Input, searched_obstacles: &mut HashSet<usize>) -> usize {
//     let mut position = Position {
//         position: obstacle,
//         direction: rotation,
//         width: input.position.width,
//         height: input.position.height,
//     };
//     position.step();
//     let deflected_direction = position.direction.rotate270();
//     let deflected_position = position.position;
//     // println!("{}", visualize_visited_time(&movement_map, input.position.width, Some(obstacle), Some((deflected_position, deflected_direction)), None, None));
//     // println!();
//     // println!("{:#06b}", *position.direction);
//     // println!("{:#06b}", *deflected_direction);
//     // println!("{:#06b}", *incoming_direction);
//     // println!("{:#06b}", movement_map[deflected_position].0);
//
//     // when a path has passed in the loop direction
//     let time = if movement_map[deflected_position].0 & *deflected_direction != 0 {
//         movement_map[deflected_position].1[deflected_direction.index()].unwrap()
//     } else {
//         return 0;
//     };
//
//     let mut added_obstacles = 0;
//
//     let passed_direction = position.direction.rotate90();
//     while let Some(position) = position.step() {
//         if input.char_map[position] == '#' {
//             // found an obstacle in this direction
//             break;
//         }
//
//         let mut obstacle_position = Position {
//             position,
//             direction: passed_direction,
//             width: input.position.width,
//             height: input.position.height,
//         };
//
//         // do not check obstacle if path goes in that direction (we'll check it anyway)
//         if movement_map[position] & *passed_direction.rotate90() == 0 {
//             if let Some(existing_obstacle) = obstacle_position.look() {
//                 if input.char_map[existing_obstacle] == '#' {
//                     #[cfg(debug_assertions)]
//                     println!("{}", visualize_paths(&input, movement_map, input.position.width, Some(obstacle), Some((deflected_position, deflected_direction)), Some(existing_obstacle), None));
//                     #[cfg(debug_assertions)]
//                     println!();
//                     if !searched_obstacles.contains(&existing_obstacle) {
//                         let mut obstacles = searched_obstacles.clone();
//                         obstacles.insert(existing_obstacle);
//                         added_obstacles += search_obstacle(start, existing_obstacle, passed_direction.rotate90().rotate90(), movement_map, input, searched_obstacles);
//                     }
//                 }
//             }
//         }
//
//         // when we passed in the opposite direction of the loop direction
//         let passed_time = if movement_map[position].0 & *passed_direction != 0 {
//             movement_map[position].1[passed_direction.index()].unwrap()
//         } else {
//             continue;
//         };
//
//         // we can skip from a later point to an earlier point
//         if passed_time > time {
//             if let Some(obstacle_position) = obstacle_position.look() {
//                 if obstacle_position != *start {
//                     added_obstacles += 1;
//                     #[cfg(debug_assertions)]
//                     println!("{}", visualize_paths(input, &movement_map, input.position.width, Some(obstacle), Some((deflected_position, deflected_direction)), Some(position), Some(obstacle_position)));
//                     #[cfg(debug_assertions)]
//                     println!();
//                 }
//             }
//         }
//     }
//
//     added_obstacles
// }

pub fn process_part2(input: &Input) -> eyre::Result<String> {
    let original_movement = movement_map(input).unwrap();
    let mut new_map = input.clone();
    let result: usize = input.char_map.iter()
        .enumerate()
        .filter(|(_, character)| **character != '#' && **character != '^')
        .filter(|(position, _)| original_movement[*position] != 0)
        .map(|(position, _)| {
            let tmp = new_map.char_map[position];
            new_map.char_map[position] = '#';
            let movement = movement_map(&new_map);
            // #[cfg(debug_assertions)]
            // match &movement {
                // Ok(map) | Err(map) => {
                //     println!("{}", visualize_paths(&new_map, &map, input.position.width, Some(position), None, None, None))
                // },
                // Err(map) => {
                    // println!("{}", visualize_paths(&new_map, &map, input.position.width, Some(position), None, None, None));
                    // println!();
                // },
                // Ok(map) => {
                //     println!("{}", visualize_paths(&new_map, &map, input.position.width, Some(position), None, None, None));
                //     println!();
                // },
                // _ => (),
            // }
            new_map.char_map[position] = tmp;
            (position, movement)
        })
        .filter_map(|(position, map)| map.err().map(|map| (position, map)))
        .count();

    Ok(result.to_string())
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
    pub fn test_example() {
        let raw_input = r"
....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...
";
        let input = raw_input.parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!("41", result);

        let result = process_part2(&input).unwrap();
        assert_eq!("6", result);

        let raw_input = r"
..........
.#........
.......#..
..........
..........
..........
....^.....
#.........
......#...
..........
";
        let input = raw_input.parse().unwrap();

        let result = process_part2(&input).unwrap();
        assert_eq!("1", result);
    }
}

use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
// use ansi_control_codes::control_sequences::CUP;
// use ansi_escape_codes::EscapeSequence::EraseScreenSequence;
use eyre::eyre;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Coordinate, ParsedGrid};

pub const DAY: Day = Day(15);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl TryFrom<char> for Direction {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '^' => Ok(Self::North),
            '>' => Ok(Self::East),
            'v' => Ok(Self::South),
            '<' => Ok(Self::West),
            _ => Err(eyre!("Invalid direction '{value}'")),
        }
    }
}

impl Direction {
    pub const fn symbol(&self) -> char {
        match self {
            Direction::North => '^',
            Direction::East => '>',
            Direction::South => 'v',
            Direction::West => '<',
        }
    }

    pub const fn rotate90(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub const fn rotate180(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    pub const fn rotate270(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }

    pub const fn vertical(&self) -> bool {
        match self {
            Direction::North | Direction::South => true,
            Direction::East | Direction::West => false,
        }
    }

    pub const fn horizontal(&self) -> bool {
        match self {
            Direction::North | Direction::South => false,
            Direction::East | Direction::West => true,
        }
    }
}

impl Into<Coordinate> for Direction {
    fn into(self) -> Coordinate {
        match self {
            Direction::North => Coordinate(0, -1),
            Direction::East => Coordinate(1, 0),
            Direction::South => Coordinate(0, 1),
            Direction::West => Coordinate(-1, 0),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Tile {
    Robot,
    Wall,
    Box,
    Empty,
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let symbol = self.symbol();
        match self {
            Self::Robot => write!(f, "{}", symbol.bright_red().bold()),
            Self::Wall => write!(f, "{}", symbol.bright_black().dimmed()),
            Self::Box => write!(f, "{}", symbol.bright_cyan().bold()),
            Self::Empty => write!(f, "{}", symbol.bright_white().bold()),
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '@' => Ok(Self::Robot),
            '#' => Ok(Self::Wall),
            'O' => Ok(Self::Box),
            '.' => Ok(Self::Empty),
            _ => Err(eyre!("Invalid tile {value}")),
        }
    }
}

impl Into<char> for Tile {
    fn into(self) -> char {
        self.symbol()
    }
}

impl Tile {
    pub const fn symbol(&self) -> char {
        match self {
            Self::Robot => '@',
            Self::Wall => '#',
            Self::Box => 'O',
            Self::Empty => '.',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Warehouse<const X_SCALE: u8> {
    map: ParsedGrid<Tile>,
    robot_position: (usize, u8),
    horizontal_offset: Vec<u8>,
}

impl Display for Warehouse<1> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.map.display(|tile, position| {
            if position == self.robot_position.0 {
                Tile::Robot.to_string()
            } else {
                tile.to_string()
            }
        }))
    }
}

impl Display for Warehouse<2> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.map.as_slice().iter()
            .zip(self.horizontal_offset.iter())
            .enumerate()
            .chunks(self.map.width())
            .into_iter()
            .map(|line| line
                .scan(0u8, |offset, (position, (tile, tile_offset))| {
                    let tile_string = match tile {
                        Tile::Empty => {
                            if position == self.robot_position.0 {
                                if self.robot_position.1 == 0 {
                                    format!("{}{}", Tile::Robot, Tile::Empty)
                                } else {
                                    format!("{}{}", Tile::Empty, Tile::Robot)
                                }
                            } else {
                                "..".bright_black().to_string()
                            }
                        },
                        Tile::Box => "[]".bright_cyan().bold().to_string(),
                        Tile::Wall => "##".bright_white().bold().to_string(),
                        Tile::Robot => unreachable!(),
                    };
                    Some(if offset == tile_offset {
                        tile_string
                    } else if *tile_offset == 1 {
                        *offset = 1;
                        format!("{}{tile_string}", if position == self.robot_position.0 { Tile::Robot } else { Tile::Empty })
                    } else if *tile == Tile::Empty {
                        *offset = 0;
                        if position == self.robot_position.0 {
                            Tile::Robot.to_string()
                        } else {
                            Tile::Empty.to_string()
                        }
                    } else {
                        panic!("Offset");
                    })
                })
                .collect::<String>()
            )
            .join("\n")
        )
    }
}

impl<const X_SCALE: u8> FromStr for Warehouse<X_SCALE> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = s.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .join("\n");
        let mut map = map.parse::<ParsedGrid<Tile>>()?;
        let robot_position = map.as_slice().iter()
            .position(|tile| *tile == Tile::Robot)
            .ok_or(eyre!("Failed to find robot"))?;
        map.as_mut_slice()[robot_position] = Tile::Empty;
        let map_size = map.as_slice().len();

        Ok(Self{
            map,
            robot_position: (robot_position, 0),
            horizontal_offset: vec![0; map_size],
        })
    }
}

impl Warehouse<1> {
    pub fn move_robot(&mut self, direction: Direction) -> Result<(), ()> {
        let move_to = self.map.offset_index(self.robot_position.0, direction.into())?;
        let mut position = self.robot_position.0;

        while let Ok(new_position) = self.map.offset_index(position, direction.into()) {
            position = new_position;
            let tile = self.map.as_slice()[position];

            if tile == Tile::Wall {
                return Err(());
            }

            if tile == Tile::Empty {
                self.map.swap(move_to, position);
                self.map.swap(self.robot_position.0, move_to);
                self.robot_position = (move_to, 0);
                return Ok(());
            }
        }

        Err(())
    }

    pub fn box_positions(&self) -> Vec<usize> {
        self.map.as_slice().iter()
            .enumerate()
            .filter(|(_, tile)| **tile == Tile::Box)
            .map(|(index, _)| self.map.index_to_coordinate(index))
            .map(|coordinate| coordinate.0 as usize + coordinate.1 as usize * 100)
            .collect()
    }
}

impl Warehouse<2> {
    pub fn offset_position(&self, (position, offset): (usize, u8), direction: Direction) -> Result<(usize, u8), ()> {
        match direction {
            Direction::North | Direction::South => {
                self.map.offset_index(position, direction.into())
                    .map(|position| (position, offset))
            }
            Direction::East => {
                if offset == 0 {
                    Ok((position, offset + 1))
                } else {
                    self.map.offset_index(position, direction.into())
                        .map(|position| (position, 0))
                }
            }
            Direction::West => {
                if offset == 1 {
                    Ok((position, offset - 1))
                } else {
                    self.map.offset_index(position, direction.into())
                        .map(|position| (position, 1))
                }
            }
        }
    }

    pub fn look(&self, position: (usize, u8), direction: Direction) -> Result<Vec<(usize, u8)>, ()> {
        match direction {
            Direction::East | Direction::West => Ok(vec![
                self.offset_position(self.offset_position(position, direction)?, direction)?
            ]),
            Direction::North | Direction::South => {
                let offset = self.offset_position(position, direction)?;
                Ok(vec![
                    offset,
                    self.offset_position(offset, direction.rotate270())?,
                    self.offset_position(offset, direction.rotate90())?,
                ])
            }
        }
    }

    pub fn look_robot(&self, direction: Direction) -> Result<Vec<(usize, u8)>, ()> {
        let offset = self.offset_position(self.robot_position, direction)?;
        match direction {
            Direction::North => {
                Ok(vec![
                    offset,
                    self.offset_position(offset, direction.rotate270())?,
                ])
            },
            Direction::East => Ok(vec![
                offset,
            ]),
            Direction::South => {
                let offset = self.offset_position(self.robot_position, direction)?;
                Ok(vec![
                    offset,
                    self.offset_position(offset, direction.rotate90())?,
                ])
            },
            Direction::West => Ok(vec![
                self.offset_position(offset, direction)?
            ]),
        }
    }

    pub fn move_robot(&mut self, direction: Direction) -> Result<(), ()> {
        let mut visited = HashSet::new();
        let mut to_move = Vec::new();
        let mut checking = VecDeque::from_iter(self.look_robot(direction)?);
        while let Some(check) = checking.pop_front() {
            let tile = self.map.as_slice()[check.0];
            if self.horizontal_offset[check.0] != check.1 || tile == Tile::Empty || visited.contains(&check) {
                continue;
            }

            if tile == Tile::Wall {
                return Err(());
            }

            visited.insert(check);
            to_move.push(check);
            checking.extend(self.look(check, direction)?);
        }

        match (direction, self.robot_position.1) {
            (Direction::North, _) | (Direction::South, _) => for &position in to_move.iter().rev() {
                let move_to = self.offset_position(position, direction).unwrap();
                self.map.swap(move_to.0, position.0);
                (self.horizontal_offset[move_to.0], self.horizontal_offset[position.0]) = (self.horizontal_offset[position.0], self.horizontal_offset[move_to.0]);
            }
            (Direction::East, 1) => for &position in to_move.iter().rev() {
                self.horizontal_offset[position.0] = 1;
            }
            (Direction::East, 0) => for &position in to_move.iter().rev() {
                let move_to = self.offset_position(position, direction).unwrap();
                self.map.swap(move_to.0, position.0);
                self.horizontal_offset[move_to.0] = 0;
                self.horizontal_offset[position.0] = 0;
            }
            (Direction::West, 1) => for &position in to_move.iter().rev() {
                self.horizontal_offset[position.0] = 0;
            }
            (Direction::West, 0) => for &position in to_move.iter().rev() {
                let move_to = self.offset_position(position, direction).unwrap();
                self.map.swap(move_to.0, position.0);
                self.horizontal_offset[move_to.0] = 1;
                self.horizontal_offset[position.0] = 0;
            }
            _ => unreachable!(),
        }

        self.robot_position = self.offset_position(self.robot_position, direction)?;
        Ok(())
    }

    pub fn box_positions(&self) -> Vec<usize> {
        self.map.as_slice().iter()
            .zip(self.horizontal_offset.iter())
            .enumerate()
            .filter(|(_, (tile, _))| **tile == Tile::Box)
            .map(|(index, (_, offset))| (offset, self.map.index_to_coordinate(index)))
            .map(|(&offset, coordinate)| (coordinate.0 as usize * 2 + offset as usize) + coordinate.1 as usize * 100)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input<const X_SCALE: u8> {
    warehouse: Warehouse<X_SCALE>,
    moves: Vec<Direction>,
}

impl<const X_SCALE: u8> FromStr for Input<X_SCALE> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (map, moves) = s.split_once("\n\n")
            .ok_or(eyre!("Could not split map from moves"))?;

        let warehouse = map.parse()?;

        let moves = moves.lines()
            .map(str::trim)
            .flat_map(str::chars)
            .map(char::try_into)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            warehouse,
            moves,
        })
    }
}

pub fn process_part1(input: &Input<1>) -> eyre::Result<usize> {
    let mut warehouse = input.warehouse.clone();

    // println!("{warehouse}\n");
    for direction in &input.moves {
        let _ = warehouse.move_robot(*direction);
        // println!("move: {direction}");
        // println!("{warehouse}\n");
    }

    Ok(warehouse.box_positions().iter().sum())
}

pub fn process_part2(input: &Input<2>) -> eyre::Result<usize> {
    let mut warehouse = input.warehouse.clone();

    // println!("{warehouse}{}", EraseScreenSequence.to_string());
    for direction in &input.moves {
        let _ = warehouse.move_robot(*direction);
        // println!("{}{warehouse}", CUP(Some(0), Some(0)));
    }

    Ok(warehouse.box_positions().iter().sum())
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

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input = raw_input.parse()?;
        debug!(?input);

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

    fn example_1_input() -> Input<1> {
        r"########
          #..O.O.#
          ##@.O..#
          #...O..#
          #.#.O..#
          #...O..#
          #......#
          ########

          <^^>>>vv<v>>v<<
          ".parse().unwrap()
    }

    fn example_2_input<const X_SCALE: u8>() -> Input<X_SCALE> {
        r"##########
          #..O..O.O#
          #......O.#
          #.OO..O.O#
          #..O@..O.#
          #O#..O...#
          #O..O..O.#
          #.OO.O.OO#
          #....O...#
          ##########

          <vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
          vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
          ><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
          <<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
          ^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
          ^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
          >^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
          <><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
          ^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
          v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^
          ".parse().unwrap()
    }

    fn example_3_input() -> Input<2> {
        r"#######
          #...#.#
          #.....#
          #..OO@#
          #..O..#
          #.....#
          #######

          <vv<<^^<<^^
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_1_part1() {
        let input = example_1_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(2028, result);
    }

    #[test]
    pub fn test_example_2_part1() {
        let input = example_2_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(10092, result);
    }

    #[test]
    pub fn test_example_3_part2() {
        let input = example_3_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(100 * 1 + 5 + 100 * 2 + 7 + 100 * 3 + 6, result);
    }

    #[test]
    pub fn test_example_2_part2() {
        let input = example_2_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(9021, result);
    }
}

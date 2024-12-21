use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Add, AddAssign, Deref, Mul, Sub, SubAssign};
use std::str::FromStr;

use eyre::anyhow;
use itertools::Itertools;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Lines<Line: FromStr + Sized + Clone + Debug + Eq + PartialEq + Hash> {
    lines: Vec<Line>,
}

impl<Line: FromStr<Err = eyre::Error> + Sized + Clone + Debug + Eq + PartialEq + Hash> FromStr for Lines<Line> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(Line::from_str)
            .collect::<Result<_, _>>()?;
        Ok(Self { lines })
    }
}

impl<Line: FromStr + Sized + Clone + Debug + Eq + PartialEq + Hash> Deref for Lines<Line> {
    type Target = [Line];

    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];
    pub const DISPLAY: [char; 16] = [
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
    ];

    pub const fn symbol(&self) -> char {
        match self {
            Self::North => '^',
            Self::East => '>',
            Self::South => 'v',
            Self::West => '<',
        }
    }

    pub const fn rotate90(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub const fn rotate180(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub const fn rotate270(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    pub const fn vertical(&self) -> bool {
        match self {
            Self::North | Self::South => true,
            Self::East | Self::West => false,
        }
    }

    pub const fn horizontal(&self) -> bool {
        match self {
            Self::North | Self::South => false,
            Self::East | Self::West => true,
        }
    }

    pub const fn mask(&self) -> u8 {
        match self {
            Direction::North => 1 << 0,
            Direction::East => 1 << 1,
            Direction::South => 1 << 2,
            Direction::West => 1 << 3,
        }
    }

    pub fn from_mask(mask: u8) -> Vec<Self> {
        (0..4).filter(|shift| (mask & (1 << *shift) != 0))
            .map(|shift| {
                match shift {
                    0 => Self::North,
                    1 => Self::East,
                    2 => Self::South,
                    3 => Self::West,
                    _ => unreachable!(),
                }
            }).collect()
    }
}

impl Into<Coordinate> for Direction {
    fn into(self) -> Coordinate {
        match self {
            Self::North => Coordinate(0, -1),
            Self::East => Coordinate(1, 0),
            Self::South => Coordinate(0, 1),
            Self::West => Coordinate(-1, 0),
        }
    }
}

// x, y
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Coordinate(pub isize, pub isize);

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl Coordinate {
    pub const NORTH: Self = Self(0, -1);
    pub const EAST: Self = Self(1, 0);
    pub const SOUTH: Self = Self(0, 1);
    pub const WEST: Self = Self(-1, 0);

    pub const CARDINALITIES: [Self; 4] = [
        Self::NORTH, // North
        Self::EAST, // East
        Self::SOUTH, // South
        Self::WEST, // West
    ];

    pub const EXTENDED_CARDINALITIES: [Self; 8] = [
        Self(0, 1), // North
        Self(1, 1), // Northeast
        Self(1, 0), // East
        Self(1, -1), // Southeast
        Self(0, -1), // South
        Self(-1, -1), // Southwest
        Self(-1, 0), // West
        Self(-1, 1), // Northwest
    ];

    pub const fn eigen_axis(self) -> Self {
        let x_direction = if self.0 == 0 {
            0
        } else {
            self.0/self.0.abs()
        };
        let y_direction = if self.1 == 0 {
            0
        } else {
            self.1/self.1.abs()
        };
        Coordinate(x_direction, y_direction)
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}


impl Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for Coordinate {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl Mul<isize> for Coordinate {
    type Output = Coordinate;

    fn mul(self, rhs: isize) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Grid {
    char_map: Vec<char>,
    width: usize,
}

struct Node {
    position: usize,
    distance: usize,
}

#[allow(dead_code)]
impl Grid {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn index_to_coordinate(&self, index: usize) -> Coordinate {
        Coordinate((index % self.width) as isize, (index / self.width) as isize)
    }

    pub fn coordinate_to_index(&self, Coordinate(x, y): Coordinate) -> Result<usize, ()> {
        if x < 0 || y < 0 || x >= self.width as isize {
            return Err(())
        }

        let index = x as usize + y as usize * self.width;
        if index >= self.char_map.len() {
            Err(())
        } else {
            Ok(index)
        }
    }

    pub fn offset_index(&self, index: usize, offset: Coordinate) -> Result<usize, ()> {
        self.coordinate_to_index(self.index_to_coordinate(index) + offset)
    }

    pub fn as_slice(&self) -> &[char] {
        &self.char_map
    }

    pub fn display<F: Fn(char, usize) -> String>(&self, postprocess: F) -> GridDisplay<F> {
        GridDisplay {
            grid: self,
            postprocess,
        }
    }

    pub fn row(&self, index: usize) -> impl Iterator<Item = &char> {
        self.char_map[index * self.width..(index + 1)*self.width()].iter()
    }

    pub fn col(&self, index: usize) -> impl Iterator<Item = &char> {
        if index > self.width {
            panic!();
        }
        self.char_map.iter().skip(index).step_by(self.width)
    }

    pub fn flood(&self, start: usize, is_wall: impl Fn(char) -> bool) -> Vec<usize> {
        let mut to_visit = VecDeque::from([Node { position: start, distance: 0 }]);
        let mut distances = vec![usize::MAX; self.char_map.len()];
        distances[start] = 0;

        while let Some(Node { position, distance }) = to_visit.pop_front() {
            let distance = distance + 1;
            for direction in Direction::ALL {
                if let Ok(position) = self.offset_index(position, direction.into()) {
                    if distance < distances[position] && !is_wall(self.char_map[position]) {
                        distances[position] = distance;
                        to_visit.push_back(Node { position, distance });
                    }
                }
            }
        }

        distances
    }
}

pub struct GridDisplay<'grid, F: Fn(char, usize) -> String> {
    grid: &'grid Grid,
    postprocess: F,
}

impl<F: Fn(char, usize) -> String> Display for GridDisplay<'_, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.grid.char_map.iter()
            .enumerate()
            .chunks(self.grid.width)
            .into_iter()
            .map(|line| line
                .map(|(index, character)| (self.postprocess)(*character, index))
                .collect::<String>()
            )
            .join("\n")
        )
    }
}

impl FromStr for Grid {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let preprocessed = s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        let width = if let Some(line) = preprocessed.first() {
            line.len()
        } else {
            return Err(anyhow!("Input is empty"));
        };

        let char_map = preprocessed.iter()
            .map(|line| line.chars())
            .flatten()
            .collect();

        Ok(Self {
            char_map,
            width
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ParsedGrid<T> {
    map: Vec<T>,
    width: usize,
}

impl<T: Default> ParsedGrid<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            map: (0..width * height).map(|_| Default::default()).collect(),
            width,
        }
    }
}

impl<T> ParsedGrid<T> {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn index_to_coordinate(&self, index: usize) -> Coordinate {
        Coordinate((index % self.width) as isize, (index / self.width) as isize)
    }

    pub fn coordinate_to_index(&self, Coordinate(x, y): Coordinate) -> Result<usize, ()> {
        if x < 0 || y < 0 || x >= self.width as isize {
            return Err(())
        }

        let index = x as usize + y as usize * self.width;
        if index >= self.map.len() {
            Err(())
        } else {
            Ok(index)
        }
    }

    pub fn offset_index(&self, index: usize, offset: Coordinate) -> Result<usize, ()> {
        self.coordinate_to_index(self.index_to_coordinate(index) + offset)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.map
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.map
    }

    pub fn display<F: Fn(&T, usize) -> D, D: Display>(&self, postprocess: F) -> ParsedGridDisplay<T, F, D> {
        ParsedGridDisplay {
            grid: self,
            postprocess,
        }
    }
}

impl<T: Copy> ParsedGrid<T> {
    pub fn swap(&mut self, a: usize, b: usize) {
        (self.map[a], self.map[b]) = (self.map[b], self.map[a])
    }
}

pub struct ParsedGridDisplay<'grid, T, F: Fn(&T, usize) -> D, D: Display> {
    grid: &'grid ParsedGrid<T>,
    postprocess: F,
}

impl<T, F: Fn(&T, usize) -> D, D: Display> Display for ParsedGridDisplay<'_, T, F, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.grid.map.iter()
            .enumerate()
            .chunks(self.grid.width)
            .into_iter()
            .map(|line| line
                .map(|(index, character)| (self.postprocess)(character, index).to_string())
                .collect::<String>()
            )
            .join("\n")
        )
    }
}

impl<T: TryFrom<char, Error = eyre::Error>> FromStr for ParsedGrid<T> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let preprocessed = s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        let width = if let Some(line) = preprocessed.first() {
            line.len()
        } else {
            return Err(anyhow!("Input is empty"));
        };

        let map = preprocessed.iter()
            .map(|line| line.chars().map(|char| char.try_into()))
            .flatten()
            .collect::<Result<_, _>>()?;

        Ok(Self {
            map,
            width
        })
    }
}

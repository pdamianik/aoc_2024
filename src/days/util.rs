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
pub struct ParsedGrid<T: TryFrom<char>> {
    map: Vec<T>,
    width: usize,
}

impl<T: TryFrom<char>> ParsedGrid<T> {
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

impl<T: TryFrom<char> + Copy> ParsedGrid<T> {
    pub fn swap(&mut self, a: usize, b: usize) {
        (self.map[a], self.map[b]) = (self.map[b], self.map[a])
    }
}

pub struct ParsedGridDisplay<'grid, T: TryFrom<char>, F: Fn(&T, usize) -> D, D: Display> {
    grid: &'grid ParsedGrid<T>,
    postprocess: F,
}

impl<T: TryFrom<char>, F: Fn(&T, usize) -> D, D: Display> Display for ParsedGridDisplay<'_, T, F, D> {
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

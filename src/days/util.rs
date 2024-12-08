use std::fmt::{Debug, Formatter};
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

impl Grid {
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

    pub fn as_slice(&self) -> &[char] {
        &self.char_map
    }

    pub fn display<F: Fn(char, usize) -> String>(&self, postprocess: F) -> GridDisplay<F> {
        GridDisplay {
            grid: self,
            postprocess,
        }
    }
}

pub struct GridDisplay<'grid, F: Fn(char, usize) -> String> {
    grid: &'grid Grid,
    postprocess: F,
}

impl<F: Fn(char, usize) -> String> std::fmt::Display for GridDisplay<'_, F> {
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

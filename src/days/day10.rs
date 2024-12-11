use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;
use eyre::anyhow;
use owo_colors::{CssColors, DynColor, OwoColorize};
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::{Coordinate, ParsedGrid};

pub const DAY: Day = Day(10);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapPosition<'map> {
    map: &'map ParsedGrid<Height>,
    position: usize,
}

impl MapPosition<'_> {
    pub fn height(&self) -> &Height {
        &self.map.as_slice()[self.position]
    }

    pub fn is_trailhead(&self) -> bool {
        *self.height() == Height::MIN
    }

    pub fn is_trail_end(&self) -> bool {
        *self.height() == Height::MAX
    }

    pub fn offset(&self, offset: Coordinate) -> Result<Self, ()> {
        let position = self.map.offset_index(self.position, offset)?;
        Ok(Self {
            map: self.map,
            position,
        })
    }

    pub fn display<C: DynColor + Copy, F: Fn(usize, &Height) -> Option<String>>(&self, color: C, postprocess: F) -> MapPositionDisplay<C, F> {
        MapPositionDisplay {
            position: self,
            color,
            postprocess,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapPositionDisplay<'input, 'position: 'input, C: DynColor + Copy, F: Fn(usize, &Height) -> Option<String>> {
    position: &'position MapPosition<'input>,
    color: C,
    postprocess: F,
}

impl<C: DynColor + Copy, F: Fn(usize, &Height) -> Option<String>> Display for MapPositionDisplay<'_, '_, C, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
               self.position.map.display(|height, index| {
                   if let Some(formatted) = (self.postprocess)(index, height) {
                       formatted
                   } else {
                       let foreground = OwoColorize::color(&**height, height.color());
                       if index == self.position.position {
                           foreground.on_color(self.color).to_string()
                       } else {
                           foreground.to_string()
                       }
                   }
               })
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputDisplay<'input, F: Fn(usize, &Height) -> Option<String>> {
    input: &'input Input,
    postprocess: F,
}

impl<F: Fn(usize, &Height) -> Option<String>> Display for InputDisplay<'_, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            self.input.map.display(|height, index| {
                if let Some(formatted) = (self.postprocess)(index, height) {
                    formatted
                } else {
                    OwoColorize::color(&**height, height.color()).to_string()
                }
            })
        )
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Height(u8);

impl Height {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(9);
}

impl Deref for Height {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<char> for Height {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0'..='9' => Ok(Self(value as u8 - '0' as u8)),
            _ => Err(anyhow!("Invalid char {value} for height"))
        }
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Height {
    const HEIGHT_COLOURS: [CssColors; 8] = [
        CssColors::LightGray,
        CssColors::HoneyDew,
        CssColors::GreenYellow,
        CssColors::Khaki,
        CssColors::NavajoWhite,
        CssColors::Orange,
        CssColors::OrangeRed,
        CssColors::Red,
    ];

    pub fn color(&self) -> CssColors {
        match self.0 {
            0..=1 => Self::HEIGHT_COLOURS[0],
            2 => Self::HEIGHT_COLOURS[1],
            3..=9 => Self::HEIGHT_COLOURS[self.0 as usize - 3],
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    map: ParsedGrid<Height>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = s.parse()?;

        Ok(Self {
            map,
        })
    }
}

impl Input {
    pub fn display<F: Fn(usize, &Height) -> Option<String>>(&self, postprocess: F) -> InputDisplay<'_, F> {
        InputDisplay {
            input: self,
            postprocess,
        }
    }

    pub fn trail_ends(&self) -> impl Iterator<Item = MapPosition> {
        self.map.as_slice().iter()
            .enumerate()
            .filter(|(_, height)| ***height == 9)
            .map(|(position, _)| MapPosition {
                map: &self.map,
                position,
            })
    }

    pub fn trail_heads(&self) -> impl Iterator<Item = MapPosition> {
        self.map.as_slice().iter()
            .enumerate()
            .filter(|(_, height)| ***height == 0)
            .map(|(position, _)| MapPosition {
                map: &self.map,
                position,
            })
    }

    pub fn position(&self, position: usize) -> MapPosition {
        MapPosition {
            map: &self.map,
            position,
        }
    }
}

pub async fn process_part1(input: Arc<Input>) -> eyre::Result<usize> {
    let scores = Arc::new((0..input.map.as_slice().len())
        .map(|_| AtomicUsize::new(0))
        .collect::<Vec<_>>());

    let handles = input.trail_ends().map(|trail_end| {
        let input = input.clone();
        let scores = scores.clone();
        let position = trail_end.position;
        tokio::spawn(async move {
            let trail_end = input.position(position);
            let mut positions = VecDeque::from_iter(std::iter::once(trail_end));
            let mut seen_position = HashSet::new();
            while let Some(current_position) = positions.pop_front() {
                if !seen_position.insert(current_position.clone()) {
                    continue;
                }
                scores[current_position.position].fetch_add(1, Ordering::Relaxed);
                if *current_position.height() == Height::MIN {
                    continue;
                }
                let new_height = current_position.height().0 - 1;

                for direction in Coordinate::CARDINALITIES {
                    if let Ok(new_position) = current_position.offset(direction) {
                        if new_position.height().0 == new_height {
                            positions.push_back(new_position);
                        }
                    }
                }
            }
        })
    }).collect::<Vec<_>>();

    for handle in handles {
        handle.await?;
    }

    let scores = scores.iter()
        .map(|score| score.load(Ordering::Relaxed))
        .collect::<Vec<_>>();

    let result = input.trail_heads()
        .map(|trail_head| scores[trail_head.position]).
        sum();

    Ok(result)
}

pub async fn process_part2(input: &Input) -> eyre::Result<usize> {
    let scores = Arc::new((0..input.map.as_slice().len())
        .map(|_| AtomicUsize::new(0))
        .collect::<Vec<_>>());

    let handles = input.trail_ends().map(|trail_end| {
        let input = input.clone();
        let scores = scores.clone();
        let position = trail_end.position;
        tokio::spawn(async move {
            let trail_end = input.position(position);
            let mut positions = VecDeque::from_iter(std::iter::once(trail_end));
            while let Some(current_position) = positions.pop_front() {
                scores[current_position.position].fetch_add(1, Ordering::Relaxed);
                if *current_position.height() == Height::MIN {
                    continue;
                }
                let new_height = current_position.height().0 - 1;

                for direction in Coordinate::CARDINALITIES {
                    if let Ok(new_position) = current_position.offset(direction) {
                        if new_position.height().0 == new_height {
                            positions.push_back(new_position);
                        }
                    }
                }
            }
        })
    }).collect::<Vec<_>>();

    for handle in handles {
        handle.await?;
    }

    let scores = scores.iter()
        .map(|score| score.load(Ordering::Relaxed))
        .collect::<Vec<_>>();

    let result = input.trail_heads()
        .map(|trail_head| scores[trail_head.position]).
        sum();

    Ok(result)
}

pub async fn run() -> eyre::Result<()> {
    let day_span = span!(Level::ERROR, "", "{}", DAY);
    async {
        info!("Running {DAY}");

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input: Input = raw_input.parse()?;
        debug!(?input);
        let input = Arc::new(input);

        let start1 = SystemTime::now();
        let result1 = process_part1(input.clone()).await?;
        let end1 = SystemTime::now();
        let start2 = SystemTime::now();
        let result2 = process_part2(&input).await?;
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
        r"89010123
          78121874
          87430965
          96549874
          45678903
          32019012
          01329801
          10456732
          ".parse().unwrap()
    }

    #[tokio::test]
    pub async fn test_example_part1() {
        let input = example_input();
        // println!("{input:?}");

        let result = process_part1(Arc::new(input)).await.unwrap();
        assert_eq!(36, result);
    }

    #[tokio::test]
    pub async fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).await.unwrap();
        assert_eq!(81, result);
    }
}

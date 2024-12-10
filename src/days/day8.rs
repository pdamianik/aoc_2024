use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::time::SystemTime;
use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;
use crate::days::util::Grid;

pub const DAY: Day = Day(8);

fn char_to_index(character: char) -> u8 {
    match character {
        'a'..='z' => character as u8 - 'a' as u8,
        'A'..='Z' => character as u8 - 'A' as u8 + 26,
        '0'..='9' => character as u8 - '0' as u8 + 52,
        _ => panic!("{character} is not a valid frequency character"),
    }
}

fn color_character(character: char) -> String {
    match char_to_index(character) % 4 {
        0 => character.green().bold().to_string(),
        1 => character.bright_red().bold().to_string(),
        2 => character.bright_yellow().bold().to_string(),
        3 => character.white().bold().to_string(),
        _ => unreachable!(),
    }
}

pub struct Antinode<'input, 'layer: 'input, 'pair: 'layer + 'input> {
    pair: &'pair Pair<'input, 'layer>,
    position: usize,
}

impl Antinode<'_, '_, '_> {
    pub fn display<F: Fn(char, usize) -> Option<String>>(&self, postprocess: F) -> AntinodeDisplay<F> {
        AntinodeDisplay {
            anitnode: self,
            postprocess,
        }
    }

    pub fn mark(&self, mask: &mut [bool]) {
        mask[self.position] = true;
    }
}

pub struct AntinodeDisplay<'input, 'layer: 'input, 'pair: 'layer + 'input, 'antinode: 'pair + 'layer + 'input, F: Fn(char, usize) -> Option<String>> {
    anitnode: &'antinode Antinode<'input, 'layer, 'pair>,
    postprocess: F,
}

impl<F: Fn(char, usize) -> Option<String>> std::fmt::Display for AntinodeDisplay<'_, '_, '_, '_, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.anitnode.pair.display(|character, index| {
            if let Some(result) = (self.postprocess)(character, index) {
                return Some(result);
            }
            if index == self.anitnode.position {
                Some('#'.yellow().on_bright_purple().to_string())
            } else {
                None
            }
        }))
    }
}

pub struct Pair<'input, 'layer: 'input> {
    layer: &'layer Layer<'input>,
    first: usize,
    second: usize,
}

impl Pair<'_, '_> {
    pub fn display<F: Fn(char, usize) -> Option<String>>(&self, postprocess: F) -> PairDisplay<F> {
        PairDisplay {
            pair: self,
            postprocess,
        }
    }

    pub fn antinodes(&self) -> Vec<Antinode<'_, '_, '_>> {
        let first_coordinate = self.layer.input.grid.index_to_coordinate(self.first);
        let second_coordinate = self.layer.input.grid.index_to_coordinate(self.second);
        [
            second_coordinate - first_coordinate + second_coordinate,
            first_coordinate - second_coordinate + first_coordinate,
        ]
            .map(|coordinate| self.layer.input.grid.coordinate_to_index(coordinate))
            .into_iter()
            .filter_map(|index| index.ok())
            .map(|position| Antinode {
                pair: self,
                position,
            })
            .collect()
    }

    pub fn antinodes2(&self) -> Vec<Antinode<'_, '_, '_>> {
        let first_coordinate = self.layer.input.grid.index_to_coordinate(self.first);
        let second_coordinate = self.layer.input.grid.index_to_coordinate(self.second);
        let mut antinodes = Vec::new();

        let difference = first_coordinate - second_coordinate;
        let mut coordinate = first_coordinate;
        while let Ok(position) = self.layer.input.grid.coordinate_to_index(coordinate) {
            antinodes.push(Antinode {
                pair: self,
                position,
            });
            coordinate += difference;
        }

        let difference = second_coordinate - first_coordinate;
        let mut coordinate = second_coordinate;
        while let Ok(position) = self.layer.input.grid.coordinate_to_index(coordinate) {
            antinodes.push(Antinode {
                pair: self,
                position,
            });
            coordinate += difference;
        }

        antinodes
    }
}

pub struct PairDisplay<'input, 'layer: 'input, 'pair: 'layer + 'input, F: Fn(char, usize) -> Option<String>> {
    pair: &'pair Pair<'input, 'layer>,
    postprocess: F,
}

impl<F: Fn(char, usize) -> Option<String>> std::fmt::Display for PairDisplay<'_, '_, '_, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pair.layer.display(|character, index| {
            if let Some(result) = (self.postprocess)(character, index) {
                return Some(result);
            }
            if index == self.pair.first || index == self.pair.second {
                Some(color_character(character).on_bright_cyan().to_string())
            } else {
                None
            }
        }))
    }
}

pub struct Layer<'input> {
    input: &'input Input,
    character: char,
}

impl Layer<'_> {
    pub fn display<F: Fn(char, usize) -> Option<String>>(&self, postprocess: F) -> LayerDisplay<F> {
        LayerDisplay {
            layer: self,
            postprocess,
        }
    }

    pub fn pairs(&self) -> Vec<Pair<'_, '_>> {
        let positions = &self.input.positions[char_to_index(self.character) as usize];
        positions.iter()
            .enumerate()
            .map(|(i, first)| {
                positions[i + 1..].iter()
                    .map(|second| (*first, *second))
            })
            .flatten()
            .map(|(first, second)| Pair {
                layer: self,
                first,
                second,
            })
            .collect()
    }
}

pub struct LayerDisplay<'input, 'layer: 'input, F: Fn(char, usize) -> Option<String>> {
    layer: &'layer Layer<'input>,
    postprocess: F,
}

impl<F: Fn(char, usize) -> Option<String>> std::fmt::Display for LayerDisplay<'_, '_, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_character = color_character(self.layer.character);
        self.layer.input.grid.display(|character, index| {
            let character = if character != self.layer.character {
                '.'
            } else {
                character
            };
            if let Some(result) = (self.postprocess)(character, index) {
                return result;
            }
            if character == '.' {
                character.dimmed().to_string()
            } else {
                formatted_character.clone()
            }
        }).fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Input {
    grid: Grid,
    characters: HashSet<char>,
    positions: [Vec<usize>; 62],
}

impl Hash for Input {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.grid.hash(state);
        self.positions.hash(state);
    }
}

impl Input {
    pub fn display<F: Fn(char, usize) -> Option<String>>(&self, postprocess: F) -> InputDisplay<F> {
        InputDisplay {
            input: self,
            postprocess,
        }
    }

    pub fn layers(&self) -> Vec<Layer<'_>> {
        self.characters.iter()
            .map(|&character| Layer {
                input: self,
                character,
            })
            .collect()
    }
}

pub struct InputDisplay<'input, F: Fn(char, usize) -> Option<String>> {
    input: &'input Input,
    postprocess: F,
}

impl<F: Fn(char, usize) -> Option<String>> std::fmt::Display for InputDisplay<'_, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.input.grid.display(|character, index| {
            if let Some(result) = (self.postprocess)(character, index) {
                return result;
            }
            if character == '.' {
                character.dimmed().to_string()
            } else {
                color_character(character)
            }
        }).fmt(f)
    }
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid: Grid = s.parse()?;
        let mut positions: [Vec<usize>; 62] = array_init::array_init(|_| Vec::new());

        for (index, character) in grid.as_slice().iter().enumerate() {
            if *character != '.' {
                positions[char_to_index(*character) as usize].push(index);
            }
        }

        let characters = grid.as_slice().iter()
            .filter(|character| **character != '.')
            .cloned()
            .collect();

        Ok(Self {
            grid,
            characters,
            positions,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let mut mask = vec![false; input.grid.as_slice().len()];
    // println!("{}\n", input.display(|_, _| None));
    for layer in input.layers() {
        // println!("{}\n", layer.display(|_, _| None));
        for pair in layer.pairs() {
            // println!("{}\n", pair.display(|_, _| None));
            for antinode in pair.antinodes() {
                // println!("{}\n", antinode.display(|_, _| None));
                antinode.mark(&mut mask);
            }
        }
    }
    let result: usize = mask.iter().filter(|has_antinode| **has_antinode).count();

    Ok(result)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let mut mask = vec![false; input.grid.as_slice().len()];
    // println!("{}\n", input.display(|_, _| None));
    for layer in input.layers() {
        // println!("{}\n", layer.display(|_, _| None));
        for pair in layer.pairs() {
            // println!("{}\n", pair.display(|_, _| None));
            for antinode in pair.antinodes2() {
                // println!("{}\n", antinode.display(|_, _| None));
                antinode.mark(&mut mask);
            }
        }
    }
    let result: usize = mask.iter().filter(|has_antinode| **has_antinode).count();

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

    fn example_input() -> Input {
        r"............
          ........0...
          .....0......
          .......0....
          ....0.......
          ......A.....
          ............
          ............
          ........A...
          .........A..
          ............
          ............
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(14, result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(34, result);
    }
}

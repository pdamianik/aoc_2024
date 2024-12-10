use std::cmp::min;
use std::str::FromStr;
use std::time::SystemTime;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(4);

fn transpose(input: &[String]) -> Vec<String> {
    (0..input[0].len())
        .map(|i| input.iter().map(|inner| inner.chars().nth(i).unwrap()).collect::<String>())
        .collect()
}

fn rotate_pos(input: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    for start_row in 0..input.len() {
        let diagonal_length = min(start_row + 1, input[0].len());
        let mut diagonal = String::with_capacity(diagonal_length);
        for offset in 0..diagonal_length {
            diagonal.push(input[start_row - offset].chars().nth(offset).unwrap());
        }
        results.push(diagonal);
    }
    for start_col in 1..input[0].len() {
        let diagonal_length = min(input[0].len() - start_col, input.len());
        let mut diagonal = String::with_capacity(diagonal_length);
        for offset in 0..diagonal_length {
            diagonal.push(input[input.len() - 1 - offset].chars().nth(start_col + offset).unwrap());
        }
        results.push(diagonal);
    }
    results
}

fn rotate_neg(input: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    for start_col in (0..input[0].len()).rev() {
        let diagonal_length = min(input[0].len() - start_col, input.len());
        let mut diagonal = String::with_capacity(diagonal_length);
        for offset in 0..diagonal_length {
            diagonal.push(input[offset].chars().nth(start_col + offset).unwrap())
        }
        results.push(diagonal);
    }

    for start_row in 1..input.len() {
        let diagonal_length = min(input.len() - start_row, input[0].len());
        let mut diagonal = String::with_capacity(diagonal_length);
        for offset in 0..diagonal_length {
            diagonal.push(input[start_row + offset].chars().nth(offset).unwrap());
        }
        results.push(diagonal);
    }
    results
}

fn find_all(text: &str, needle: &str) -> Vec<usize> {
    let mut text = text;
    let mut indices = Vec::new();

    while let Some(offset) = text.find(needle) {
        indices.push(indices.last().map(|&val| val + needle.len()).unwrap_or(0) + offset);
        text = &text[offset + needle.len()..];
    }

    indices
}

fn search(input: &[String]) -> [Vec<(usize, usize)>; 2] {
    let occurrences = input.iter()
        .enumerate()
        .map(|(rowi, row)|
            find_all(row, "XMAS").iter()
                .map(|&coli| (rowi, coli))
                .collect::<Vec<_>>()
        )
        .flatten()
        .collect();

    let reverse = input.iter()
        .enumerate()
        .map(|(rowi, row)|
            find_all(row, "SAMX").iter()
                .map(|&coli| (rowi, coli + 3))
                .collect::<Vec<_>>()
        )
        .flatten()
        .collect();

    [occurrences, reverse]
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Input {
    lines: Vec<String>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        Ok(Self {
            lines,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<String> {
    let diagonal_ne = rotate_pos(&input.lines);

    let cols = transpose(&input.lines);
    let diagonal_se = rotate_neg(&input.lines);

    let [east, west] = search(&input.lines);
    let [north_east, south_west] = search(&diagonal_ne);

    let [south, north] = search(&cols);
    let [south_east, north_west] = search(&diagonal_se);

    // println!("{}", visualize1(&input, &north, &north_east, &east, &south_east, &south, &south_west, &west, &north_west));

    let result = [north, north_east, east, south_east, south, south_west, west, north_west].iter()
        .map(|occurrences| occurrences.len())
        .sum::<usize>();

    Ok(result.to_string())
}

#[allow(dead_code)]
fn visualize1(text: &[&str],
              north: &[(usize, usize)],
              north_east: &[(usize, usize)],
              east: &[(usize, usize)],
              south_east: &[(usize, usize)],
              south: &[(usize, usize)],
              south_west: &[(usize, usize)],
              west: &[(usize, usize)],
              north_west: &[(usize, usize)],
) -> String {
    let cols = text[0].len();
    let rows = text.len();
    let mut marked = vec![false; cols * rows];

    for &(col, row) in north {
        for offset in 0..4 {
            marked[(row - offset) * cols + col] = true;
        }
    }

    for &(rowi, coli) in north_east {
        let row = if rowi < rows {
            rowi - coli
        } else {
            rows - 1 - coli
        };
        let col = if rowi < rows {
            coli
        } else {
            rowi - rows + 1 + coli
        };
        for offset in 0..4 {
            marked[(row - offset) * cols + col + offset] = true;
        }
    }

    for &(row, col) in east {
        for offset in 0..4 {
            marked[row * cols + col + offset] = true;
        }
    }

    for &(rowi, coli) in south_east {
        let row = if rowi < cols {
            coli
        } else {
            rowi + 1 - cols + coli
        };
        let col = if rowi < rows {
            cols - 1 - rowi + coli
        } else {
            coli
        };
        for offset in 0..4 {
            marked[(row + offset) * cols + col + offset] = true;
        }
    }

    for &(col, row) in south {
        for offset in 0..4 {
            marked[(row + offset) * cols + col] = true;
        }
    }

    for &(rowi, coli) in south_west {
        let row = if rowi < rows {
            rowi - coli
        } else {
            rows - 1 - coli
        };
        let col = if rowi < rows {
            coli
        } else {
            rowi - rows + 1 + coli
        };
        for offset in 0..4 {
            marked[(row + offset) * cols + col - offset] = true;
        }
    }

    for &(row, col) in west {
        for offset in 0..4 {
            marked[row * cols + col - offset] = true;
        }
    }

    for &(rowi, coli) in north_west {
        let row = if rowi < cols {
            coli
        } else {
            rowi + 1 - cols + coli
        };
        let col = if rowi < rows {
            cols - 1 - rowi + coli
        } else {
            coli
        };
        for offset in 0..4 {
            marked[(row - offset) * cols + col - offset] = true;
        }
    }

    text.iter()
        .zip(marked.iter().chunks(cols).into_iter())
        .map(|(row, marks)| {
            row.chars().zip(marks)
                .map(|(character, &mark)| {
                    if mark {
                        character.bold().bright_green().to_string()
                    } else {
                        character.dimmed().to_string()
                    }
                })
                .collect::<String>()
        })
        .join("\n")
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

impl Direction {
    const ALL: [Self; 4] = [
        Self::NorthEast,
        Self::SouthEast,
        Self::SouthWest,
        Self::NorthWest,
    ];

    fn opposite(&self) -> Self {
        match self {
            Self::NorthEast => Self::SouthWest,
            Self::SouthEast => Self::NorthWest,
            Self::SouthWest => Self::NorthEast,
            Self::NorthWest => Self::SouthEast,
        }
    }

    fn offset(&self, row: usize, col: usize) -> (usize, usize) {
        assert!(row > 0);
        assert!(col > 0);
        match self {
            Self::NorthEast => (row - 1, col + 1),
            Self::SouthEast => (row + 1, col + 1),
            Self::SouthWest => (row + 1, col - 1),
            Self::NorthWest => (row - 1, col - 1),
        }
    }
}

fn check_cross(chars: &[char], _rows: usize, cols: usize, row: usize, col: usize) -> bool {
    if chars[row * cols + col] != 'A' {
        return false;
    }
    let mut count = 0;
    for direction in Direction::ALL {
        let (offset_row, offset_col) = direction.offset(row, col);
        if chars[offset_row * cols + offset_col] == 'M' {
            let (offset_row, offset_col) = direction.opposite().offset(row, col);
            if chars[offset_row * cols + offset_col] == 'S' {
                count += 1;
            }
        }
    }
    count == 2
}

pub fn process_part2(input: &Input) -> eyre::Result<String> {
    let rows = input.lines.len();
    let cols = input.lines[0].len();
    let chars = input.lines.iter()
        .map(|line| line.chars())
        .flatten()
        .collect::<Vec<_>>();

    let mut result = Vec::new();
    for row in 1..rows - 1 {
        for col in 1..cols - 1 {
            if check_cross(&chars, rows, cols, row, col) {
                result.push((row, col));
            }
        }
    }

    // println!("{}", visualize2(input, &result));

    Ok(result.len().to_string())
}

#[allow(dead_code)]
fn visualize2(text: &[&str], positions: &[(usize, usize)]) -> String {
    let cols = text[0].len();
    let rows = text.len();
    let mut marked = vec![false; cols * rows];

    for &(row, col) in positions {
        marked[row * cols + col] = true;
        marked[(row - 1) * cols + col + 1] = true;
        marked[(row + 1) * cols + col + 1] = true;
        marked[(row + 1) * cols + col - 1] = true;
        marked[(row - 1) * cols + col - 1] = true;
    }

    text.iter()
        .zip(marked.iter().chunks(cols).into_iter())
        .map(|(row, marks)| {
            row.chars().zip(marks)
                .map(|(character, &mark)| {
                    if mark {
                        character.bold().bright_green().to_string()
                    } else {
                        character.dimmed().to_string()
                    }
                })
                .collect::<String>()
        })
        .join("\n")
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
        let raw_input = r#"MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX
"#;
        let input = raw_input.parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!("18", result);

        let result = process_part2(&input).unwrap();
        assert_eq!("9", result);
    }
}

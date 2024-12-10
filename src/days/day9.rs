use std::cmp::min;
use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

const DAY: Day = Day(9);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileBlock {
    id: usize,
    index: usize,
    len: u8,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EmptyBlock {
    index: usize,
    len: u8,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Input {
    block_layout: Vec<(usize, u8)>,
    file_blocks: Vec<FileBlock>,
    empty_blocks: Vec<EmptyBlock>,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let block_layout: Vec<(usize, u8)> = s.lines()
            .map(|line| line.trim())
            .find(|line| !line.is_empty())
            .map(|line| line.chars()
                .map(|char| char.to_digit(10).unwrap() as u8)
                .scan(0, |index, len| {
                    let result = *index;
                    *index += len as usize;
                    Some((result, len))
                })
                .collect()
            )
            .ok_or(eyre!("Failed to find block layout"))?;

        let file_blocks = block_layout.iter()
            .step_by(2)
            .enumerate()
            .map(|(id, &(index, len))| FileBlock { id, index, len })
            .collect();
        let empty_blocks = block_layout.iter()
            .skip(1)
            .step_by(2)
            .map(|&(index, len)| EmptyBlock { index, len })
            .collect();

        Ok(Self {
            block_layout,
            file_blocks,
            empty_blocks,
        })
    }
}

impl Input {
    pub fn fill_holes(&self) -> Vec<usize> {
        let file_block_count = self.file_blocks.iter().map(|block| block.len as usize).sum::<usize>();
        let mut result = Vec::with_capacity(file_block_count);

        let mut insert_block = self.file_blocks.len() - 1;
        let mut insert_count = self.file_blocks[insert_block].len as usize;
        let mut index = 0;
        while result.len() < file_block_count {
            let count = min(self.file_blocks[index].len as usize, file_block_count - result.len());
            result.extend(std::iter::repeat_n(index, count));

            if result.len() >= file_block_count {
                break;
            }

            let mut empty_count = min(self.empty_blocks[index].len as usize, file_block_count - result.len());
            while empty_count > insert_count {
                result.extend(std::iter::repeat_n(insert_block, insert_count));
                empty_count -= insert_count;
                insert_block -= 1;
                insert_count = self.file_blocks[insert_block].len as usize;
            }

            result.extend(std::iter::repeat_n(insert_block, empty_count));
            if insert_count == empty_count {
                insert_block -= 1;
                insert_count = self.file_blocks[insert_block].len as usize;
            } else {
                insert_count -= empty_count;
            }
            index += 1;
        }

        result
    }
}

fn sum_range(start: usize, end: usize) -> usize {
    (end - start + 1) * (start + end) / 2
    // (end * end - start * start + start + end) / 2
    // (start..=end).sum::<usize>()
}

pub fn process_part1(input: &Input) -> eyre::Result<usize> {
    let filled = input.fill_holes();

    let checksum = filled.iter()
        .enumerate()
        .map(|(index, &val)| index * val)
        .sum();

    Ok(checksum)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let mut filler_sizes: [_; 9] = array_init::array_init(|_| vec![]);
    // let mut filler_max = [None; 9];
    for block in &input.file_blocks {
        if block.len != 0 {
            filler_sizes[block.len as usize - 1].push(block.clone());
        }
    }

    let mut checksum = 0;
    for empty in &input.empty_blocks {
        let mut space = empty.len;

        if space == 0 {
            continue;
        }

        while let Some(filler) = filler_sizes[0..space as usize]
            .iter_mut()
            .filter(|filler| filler.len() > 0)
            .filter(|filler| filler.last().unwrap().index > empty.index)
            .max_by(|a, b| a.last().unwrap().id.cmp(&b.last().unwrap().id)) {
            // println!("Trying to fill {empty:?} with space {space}");
            let filler = filler.pop().unwrap();

            let index = empty.index + (empty.len - space) as usize;
            // println!("Repositioning {filler:?} to @{index}");
            checksum += filler.id * sum_range(index, index + filler.len as usize - 1);
            space -= filler.len;

            // sanity check
            if space == 0 {
                break;
            }
        }
    }

    let filler_sum = filler_sizes.iter()
        .map(|filler|
            filler.iter().map(|filler|
                filler.id * sum_range(filler.index, filler.index + filler.len as usize - 1)
            )
                .sum::<usize>()
        )
        .sum::<usize>();

    let result: usize = checksum + filler_sum;
    // assert_ne!(result, 8518174061514, "Known to be wrong");

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
        r"2333133121414131402".parse().unwrap()
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(1928, result);
    }

    #[test]
    pub fn test_custom_part1() {
        // 0..222
        // 0222
        // 0222
        let input = "11013".parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!(0 + (1..4).map(|i| i * 2).sum::<usize>(), result);

        // 0.........111112223333
        // 03333.....11111222
        // 03333222..11111
        // 0333322211111
        let input = "1950304".parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!([0, 3, 3, 3, 3, 2, 2, 2, 1, 1, 1, 1, 1].iter().enumerate().map(|(index, &val)| index * val).sum::<usize>(), result);

        // 0..1..2
        // 02.1
        // 021
        let input = "12121".parse().unwrap();

        let result = process_part1(&input).unwrap();
        assert_eq!([0, 2, 1].iter().enumerate().map(|(index, &val)| index * val).sum::<usize>(), result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_input();

        let result = process_part2(&input).unwrap();
        assert_eq!(2858, result);
    }

    #[test]
    pub fn test_custom_part2() {
        let input = "001".parse().unwrap();
        let result = process_part2(&input).unwrap();
        assert_eq!(0, result);

        let input = "0630201".parse().unwrap();
        let result = process_part2(&input).unwrap();
        assert_eq!(18, result);

        // input from https://www.reddit.com/r/adventofcode/comments/1hajykk/2024_day_9_part_2_cant_get_part_2_to_work_any/
        let input = "12235".parse().unwrap();
        let result = process_part2(&input).unwrap();
        assert_eq!(1 + 2 + (8..=12).sum::<usize>() * 2, result);
    }

    #[test]
    pub fn test_evil_part2() {
        // input from https://www.reddit.com/r/adventofcode/comments/1haauty/2024_day_9_part_2_bonus_test_case_that_might_make/
        let input = include_str!("day9_evil1.in").parse().unwrap();
        let result = process_part2(&input).unwrap();
        assert_eq!(97898222299196, result);

        let input = include_str!("day9_evil2.in").parse().unwrap();
        let result = process_part2(&input).unwrap();
        assert_eq!(5799706413896802, result);
    }
}

use std::ops::Mul;
use std::str::FromStr;
use std::time::SystemTime;
use eyre::eyre;
use itertools::Itertools;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(14);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Robot<const WIDTH: usize, const HEIGHT: usize> {
    position: (usize, usize),
    velocity: (isize, isize),
}

impl<const WIDTH: usize, const HEIGHT: usize> Robot<WIDTH, HEIGHT> {
    pub fn patrol(&mut self, seconds: usize) {
        self.position.0 = ((self.position.0 + WIDTH) as isize + ((self.velocity.0 * seconds as isize) % WIDTH as isize)) as usize % WIDTH;
        self.position.1 = ((self.position.1 + HEIGHT) as isize + ((self.velocity.1 * seconds as isize) % HEIGHT as isize)) as usize % HEIGHT;
    }

    pub fn patrol_once(&mut self) {
        self.position.0 = ((self.position.0 + WIDTH) as isize + self.velocity.0) as usize % WIDTH;
        self.position.1 = ((self.position.1 + HEIGHT) as isize + self.velocity.1) as usize % HEIGHT;
    }

    pub fn quadrant(&self) -> Option<u8> {
        match (self.position.0 < WIDTH / 2, self.position.0 > WIDTH / 2, self.position.1 < HEIGHT / 2, self.position.1 > HEIGHT / 2) {
            (false, false, _, _) => None,
            (_, _, false, false) => None,
            (_, right, _, bottom) => {
                Some(if right { 1 } else { 0 } + if bottom { 2 } else { 0 })
            }
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> FromStr for Robot<WIDTH, HEIGHT> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (coordinate, velocity) = s.split_once(" ")
            .ok_or(eyre!("Failed to split robot into position and velocity"))?;

        let coordinate = coordinate.strip_prefix("p=")
            .ok_or(eyre!("position should be given as p=x,y"))?;
        let (x, y) = coordinate.split_once(",")
            .ok_or(eyre!("position should be given as p=x,y"))?;
        let position = (x.parse()?, y.parse()?);

        let velocity = velocity.strip_prefix("v=")
            .ok_or(eyre!("velocity should be given as v=x,y"))?;
        let (dx, dy) = velocity.split_once(",")
            .ok_or(eyre!("velocity should be given as v=x,y"))?;
        let (dx, dy) = (dx.parse::<isize>()?, dy.parse::<isize>()?);
        let velocity = (dx % WIDTH as isize, dy % HEIGHT as isize);

        Ok(Self {
            position,
            velocity,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input<const WIDTH: usize, const HEIGHT: usize> {
    robots: Vec<Robot<WIDTH, HEIGHT>>,
}

impl<const WIDTH: usize, const HEIGHT: usize> FromStr for Input<WIDTH, HEIGHT> {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let robots = s.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            robots,
        })
    }
}

pub fn process_part1<const WIDTH: usize, const HEIGHT: usize>(input: &Input<WIDTH, HEIGHT>) -> eyre::Result<usize> {
    let mut input = input.robots
        .iter().cloned()
        .collect::<Vec<_>>();

    let quadrant_counts = input.iter_mut()
        .filter_map(|robot| {
            robot.patrol(100);
            robot.quadrant()
        })
        .counts();

    Ok(quadrant_counts.values().fold(1, usize::mul))
}

fn std_deviation(data: &[usize]) -> f32 {
    let sum = data.iter().sum::<usize>() as f32;
    let count = data.len() as f32;
    let mean = sum / count;
    let variance = data.iter()
        .map(|&value| {
            let distance = mean - value as f32;
            distance * distance
        })
        .sum::<f32>() / count;

    variance.sqrt()
}

fn find_image<const WIDTH: usize, const HEIGHT: usize>(robots: &mut [Robot<WIDTH, HEIGHT>]) -> usize {
    let mut seconds = 0;
    loop {
        robots.iter_mut().for_each(|robot| robot.patrol_once());
        seconds += 1;

        let (xs, ys): (Vec<usize>, Vec<usize>) = robots.iter()
            .map(|robot| robot.position)
            .unzip();

        let x_score = std_deviation(&xs);
        let y_score = std_deviation(&ys);

        if x_score < 25.0 && y_score < 25.0 {
            return seconds;
        }
    }
}

pub fn process_part2<const WIDTH: usize, const HEIGHT: usize>(input: &Input<WIDTH, HEIGHT>) -> eyre::Result<usize> {
    let mut robots = input.robots
        .iter().cloned()
        .collect::<Vec<_>>();

    let seconds = find_image(&mut robots);
    let seconds1 = find_image(&mut robots);
    let seconds2 = find_image(&mut robots);

    if seconds1 == seconds2 {
        Ok(seconds)
    } else {
        Err(eyre!("Failed to determine"))
    }
}

pub async fn run() -> eyre::Result<()> {
    let day_span = span!(Level::ERROR, "", "{}", DAY);
    async {
        info!("Running {DAY}");

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input: Input<101, 103> = raw_input.parse()?;
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

    fn example_input() -> Input<11, 7> {
        r"p=0,4 v=3,-3
          p=6,3 v=-1,-3
          p=10,3 v=-1,2
          p=2,0 v=2,-1
          p=0,0 v=1,3
          p=3,0 v=-2,-2
          p=7,6 v=-1,-3
          p=3,0 v=-1,-2
          p=9,3 v=2,3
          p=7,3 v=-1,2
          p=2,4 v=2,-3
          p=9,5 v=-3,-3
          ".parse().unwrap()
    }

    #[test]
    pub fn test_patrol() {
        let mut robot: Robot<11, 7> = Robot {
            position: (2, 4),
            velocity: (2, -3),
        };
        assert_eq!(-3 % 7, -3);
        robot.patrol(1);
        assert_eq!(robot.position, (4, 1));
    }

    #[test]
    pub fn test_example_part1() {
        let input = example_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(12, result);
    }
}

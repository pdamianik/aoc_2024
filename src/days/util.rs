use std::fmt::Debug;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lines<Line: FromStr + Sized + Clone + Debug + Eq + PartialEq> {
    lines: Vec<Line>,
}

impl<Line: FromStr<Err = eyre::Error> + Sized + Clone + Debug + Eq + PartialEq> FromStr for Lines<Line> {
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

impl<Line: FromStr + Sized + Clone + Debug + Eq + PartialEq> Deref for Lines<Line> {
    type Target = [Line];

    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}

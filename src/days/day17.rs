use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use eyre::eyre;
use itertools::Itertools;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

pub const DAY: Day = Day(17);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ComboOperand {
    Literal(u8),
    A,
    B,
    C,
    Reserved,
}

impl From<u8> for ComboOperand {
    fn from(value: u8) -> Self {
        match value {
            0..=3 => Self::Literal(value),
            4 => Self::A,
            5 => Self::B,
            6 => Self::C,
            7 => panic!("reserved combo operand"),
            _ => panic!("invalid combo operand (must be 3 bit)"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Instruction {
    Adv(ComboOperand),
    Bxl(u8),
    Bst(ComboOperand),
    Jnz(u8),
    Bxc(u8),
    Out(ComboOperand),
    Bdv(ComboOperand),
    Cdv(ComboOperand),
}

impl Instruction {
    pub fn new(instruction: u8, operand: u8) -> Self {
        match instruction {
            0 => Self::Adv(operand.into()),
            1 => Self::Bxl(operand),
            2 => Self::Bst(operand.into()),
            3 => Self::Jnz(operand),
            4 => Self::Bxc(operand),
            5 => Self::Out(operand.into()),
            6 => Self::Bdv(operand.into()),
            7 => Self::Cdv(operand.into()),
            _ => panic!("invalid instruction {instruction} (must be 3 bit long)"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Computer {
    register_a: usize,
    register_b: usize,
    register_c: usize,
    instruction_pointer: usize,
    program: Arc<Vec<u8>>,
}

impl FromStr for Computer {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (registers, program) = s.split_once("\n\n")
            .ok_or(eyre!("Failed to split registers and program"))?;

        let mut register_a: Option<usize> = None;
        let mut register_b: Option<usize> = None;
        let mut register_c: Option<usize> = None;
        registers.lines()
            .map(|line| line.strip_prefix("Register ").unwrap())
            .map(|line| line.split_once(": ").unwrap())
            .map(|(register, value)| (register, value.parse().unwrap()))
            .for_each(|(register, value)| match register {
                "A" => register_a = Some(value),
                "B" => register_b = Some(value),
                "C" => register_c = Some(value),
                _ => panic!("invalid register name"),
            });

        let program = program.strip_prefix("Program: ")
            .ok_or(eyre!("Program should be given after \"Program: \""))?
            .trim()
            .split(",")
            .map(|value| value.parse().unwrap())
            .collect();
        let program = Arc::new(program);

        Ok(Self {
            register_a: register_a.unwrap(),
            register_b: register_b.unwrap(),
            register_c: register_c.unwrap(),
            instruction_pointer: 0,
            program,
        })
    }
}

impl Computer {
    pub fn execute(&mut self) -> Vec<u8> {
        let mut result = Vec::new();
        while self.instruction_pointer < self.program.len() {
            if let Some(output) = self.step() {
                result.push(output);
            }
            // println!("{}", self.register_a);
        }
        result
    }

    fn evaluate_combo(&self, combo_operand: ComboOperand) -> usize {
        match combo_operand {
            ComboOperand::Literal(literal) => literal as usize,
            ComboOperand::A => self.register_a,
            ComboOperand::B => self.register_b,
            ComboOperand::C => self.register_c,
            ComboOperand::Reserved => unimplemented!(),
        }
    }

    pub fn fetch(&mut self) -> Instruction {
        let instruction = self.program[self.instruction_pointer];
        let operand = self.program[self.instruction_pointer + 1];
        self.instruction_pointer += 2;
        Instruction::new(instruction, operand)
    }

    pub fn decode_all(&self) -> Vec<Instruction> {
        self.program.iter()
            .tuples()
            .map(|(&instruction, &operand)| Instruction::new(instruction, operand))
            .collect()
    }

    pub fn step(&mut self) -> Option<u8> {
        match self.fetch() {
            Instruction::Adv(operand) => {
                self.register_a >>= self.evaluate_combo(operand);
                None
            }
            Instruction::Bxl(operand) => {
                self.register_b ^= operand as usize;
                None
            }
            Instruction::Bst(operand) => {
                self.register_b = self.evaluate_combo(operand) & 0b111;
                None
            }
            Instruction::Jnz(operand) => {
                if self.register_a != 0 {
                    self.instruction_pointer = operand as usize;
                }
                None
            }
            Instruction::Bxc(_) => {
                self.register_b ^= self.register_c;
                None
            }
            Instruction::Out(operand) => {
                Some((self.evaluate_combo(operand) % 8) as u8)
            }
            Instruction::Bdv(operand) => {
                self.register_b = self.register_a >> self.evaluate_combo(operand);
                None
            }
            Instruction::Cdv(operand) => {
                self.register_c = self.register_a >> self.evaluate_combo(operand);
                None
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Input {
    computer: Computer,
}

impl FromStr for Input {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.lines()
            .map(str::trim)
            .join("\n");

        let computer = s.parse()?;

        Ok(Self {
            computer,
        })
    }
}

pub fn process_part1(input: &Input) -> eyre::Result<Vec<u8>> {
    let mut computer = input.computer.clone();

    let output = computer.execute();

    Ok(output)
}

pub fn process_part2(input: &Input) -> eyre::Result<usize> {
    let mut computer = input.computer.clone();
    let mut correct_inputs = (0..1 << 3).collect::<Vec<_>>();
    let mut new_correct_inputs = Vec::with_capacity(1 << 3);
    for (index, &expected) in input.computer.program.iter().enumerate().rev() {
        for &correct_input in &correct_inputs {
            for new_input in 0usize..1 << 3 {
                let input = (correct_input << 3) | new_input;
                computer.register_a = input;
                computer.register_b = 0;
                computer.register_c = 0;
                computer.instruction_pointer = 0;
                while computer.instruction_pointer < computer.program.len() {
                    if let Some(output) = computer.step() {
                        if expected == output {
                            if index == 0 {
                                computer.register_a = input;
                                computer.register_b = 0;
                                computer.register_c = 0;
                                computer.instruction_pointer = 0;
                                return Ok(input)
                            }
                            new_correct_inputs.push(input);
                        }
                        break;
                    }
                }
            }
        }
        std::mem::swap(&mut correct_inputs, &mut new_correct_inputs);
        new_correct_inputs.clear();
    }
    Ok(*correct_inputs.first().unwrap())
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
        println!("  part 1: {} in {:?}", result1.iter().join(","), end1.duration_since(start1).unwrap());
        println!("  part 2: {result2} in {:?}", end2.duration_since(start2).unwrap());
        Ok(())
    }
        .instrument(day_span.or_current())
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    fn example_1_input() -> Input {
        r"Register A: 729
          Register B: 0
          Register C: 0

          Program: 0,1,5,4,3,0
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_1_part1() {
        let input = example_1_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(vec![4,6,3,5,6,3,5,2,1,0], result);
    }

    fn example_2_input() -> Input {
        r"Register A: 117440
          Register B: 0
          Register C: 0

          Program: 0,3,5,4,3,0
          ".parse().unwrap()
    }

    #[test]
    pub fn test_example_2_part1() {
        let input = example_2_input();

        let result = process_part1(&input).unwrap();
        assert_eq!(vec![0,3,5,4,3,0], result);
    }

    #[test]
    pub fn test_example_part2() {
        let input = example_2_input();
        let result = process_part2(&input).unwrap();
        assert_eq!(117440, result);
    }
}

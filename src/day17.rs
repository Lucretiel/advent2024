use std::fmt::Display;

use enum_map::{EnumMap, enum_map};
use itertools::{EitherOrBoth, Itertools};
use joinery::{Joinable, separators::Comma};
use nom::{
    Parser,
    branch::alt,
    character::complete::{char, digit1, multispace0, multispace1, space0},
    combinator::eof,
};
use nom_supreme::{
    ParserExt, error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
    tag::complete::tag,
};

use crate::{library::ITResult, parser};

mod cpu {
    use std::fmt::Display;

    use enum_map::{Enum, EnumMap};
    use lazy_format::lazy_format;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub enum Code {
        Zero,
        One,
        Two,
        Three,
        Four,
        Five,
        Six,
        Seven,
    }

    impl Code {
        fn literal(self) -> usize {
            self as usize
        }

        fn combo(self, registers: &EnumMap<Register, usize>) -> usize {
            use Code::*;

            match self {
                Zero => 0,
                One => 1,
                Two => 2,
                Three => 3,
                Four => registers[Register::A],
                Five => registers[Register::B],
                Six => registers[Register::C],
                Seven => panic!("invalid combo"),
            }
        }

        fn from_value(value: usize) -> Self {
            match value & 0b111 {
                0 => Self::Zero,
                1 => Self::One,
                2 => Self::Two,
                3 => Self::Three,
                4 => Self::Four,
                5 => Self::Five,
                6 => Self::Six,
                7 => Self::Seven,
                _ => unreachable!(),
            }
        }

        pub fn describe_literal(self) -> impl Display {
            self.literal()
        }

        pub fn describe_combo(self) -> impl Display {
            use Code::*;

            lazy_format!(match (self) {
                Zero => "0",
                One => "1",
                Two => "2",
                Three => "3",
                Four => "A",
                Five => "B",
                Six => "C",
                Seven => "ERROR",
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Instruction {
        Adv,
        Bxl,
        Bst,
        Jnz,
        Bxc,
        Out,
        Bdv,
        Cdv,
    }

    impl Instruction {
        fn from_code(code: Code) -> Self {
            match code {
                Code::Zero => Self::Adv,
                Code::One => Self::Bxl,
                Code::Two => Self::Bst,
                Code::Three => Self::Jnz,
                Code::Four => Self::Bxc,
                Code::Five => Self::Out,
                Code::Six => Self::Bdv,
                Code::Seven => Self::Cdv,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
    pub enum Register {
        A,
        B,
        C,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum MachineState {
        Output(Code),
        Halt,
    }

    impl MachineState {
        pub fn output(self) -> Option<Code> {
            match self {
                Self::Output(code) => Some(code),
                Self::Halt => None,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Machine<'a> {
        registers: EnumMap<Register, usize>,
        program: &'a [Code],
        instruction_pointer: usize,
    }

    impl<'a> Machine<'a> {
        pub fn new(registers: EnumMap<Register, usize>, program: &'a [Code]) -> Self {
            Self {
                registers,
                program,
                instruction_pointer: 0,
            }
        }

        pub fn reinit(&mut self, value: usize) {
            self.registers[Register::A] = value;
        }

        fn load_instruction(&self) -> Option<(Instruction, Code)> {
            let &code = self.program.get(self.instruction_pointer)?;
            let &param = self.program.get(self.instruction_pointer + 1)?;

            let instruction = Instruction::from_code(code);
            Some((instruction, param))
        }

        fn div(&mut self, code: Code, dest: Register) {
            let lhs = self.registers[Register::A];
            let rhs = code.combo(&self.registers);

            let out = lhs >> rhs;
            self.registers[dest] = out;
        }

        fn xor_with_b(&mut self, value: usize) {
            let lhs = self.registers[Register::B];
            let out = lhs ^ value;
            self.registers[Register::B] = out;
        }

        pub fn step(&mut self) -> Option<MachineState> {
            let Some((instruction, param)) = self.load_instruction() else {
                return Some(MachineState::Halt);
            };

            let mut out = None;

            match instruction {
                Instruction::Adv => self.div(param, Register::A),
                Instruction::Bdv => self.div(param, Register::B),
                Instruction::Cdv => self.div(param, Register::C),

                Instruction::Bxl => self.xor_with_b(param.literal()),
                Instruction::Bxc => self.xor_with_b(self.registers[Register::C]),

                Instruction::Bst => {
                    self.registers[Register::B] = param.combo(&self.registers) & 0b111
                }

                // Handled later, during IP update
                Instruction::Jnz => {}
                Instruction::Out => {
                    out = Some(Code::from_value(param.combo(&self.registers)));
                }
            }

            self.instruction_pointer = match instruction {
                Instruction::Jnz if self.registers[Register::A] != 0 => param.literal(),
                _ => self.instruction_pointer + 2,
            };

            out.map(MachineState::Output)
        }

        pub fn run_until_state(&mut self) -> MachineState {
            loop {
                if let Some(state) = self.step() {
                    break state;
                }
            }
        }

        pub fn run_iter(&mut self) -> impl Iterator<Item = Code> {
            std::iter::from_fn(move || self.run_until_state().output())
        }

        #[expect(dead_code)]
        pub fn describe(&self) -> impl Display {
            lazy_format!(
                ("{}\n", lazy_format!(match (Instruction::from_code(*instruction)) {
                    Instruction::Adv => ("A >> {} -> A", code.describe_combo()),
                    Instruction::Bdv => ("A >> {} -> B", code.describe_combo()),
                    Instruction::Cdv => ("A >> {} -> C", code.describe_combo()),

                    Instruction::Bxl => ("B ^ {} -> B", code.describe_literal()),
                    Instruction::Bxc => "B ^ C -> B",

                    Instruction::Bst => ("{} & 0b111 -> B", code.describe_combo()),

                    Instruction::Jnz => ("Jump to {} if A != 0", code.describe_literal()),
                    Instruction::Out => ("Output {}", code.describe_combo()),
                })
                ) for [instruction, code] in self.program.array_chunks()
            )
        }
    }
}

fn parse_code(input: &str) -> ITResult<&str, cpu::Code> {
    use cpu::Code::*;

    alt((
        char('0').value(Zero),
        char('1').value(One),
        char('2').value(Two),
        char('3').value(Three),
        char('4').value(Four),
        char('5').value(Five),
        char('6').value(Six),
        char('7').value(Seven),
    ))
    .parse(input)
}

fn register_parser<'i>(id: char) -> impl Parser<&'i str, usize, ErrorTree<&'i str>> {
    tag("Register ")
        .terminated(char(id))
        .terminated(tag(": "))
        .precedes(digit1)
        .parse_from_str_cut()
}

fn parse_registers(input: &str) -> ITResult<&str, enum_map::EnumMap<cpu::Register, usize>> {
    use cpu::Register;

    parser! {
        register_parser('A') => a,
        multispace0,
        register_parser('B') => b,
        multispace0,
        register_parser('C') => c;

        enum_map! {
            Register::A => a,
            Register::B => b,
            Register::C => c,
        }
    }
    .parse(input)
}

fn parse_program(input: &str) -> ITResult<&str, Vec<cpu::Code>> {
    collect_separated_terminated(
        parse_code,
        char(',').delimited_by(space0),
        multispace0.terminated(eof),
    )
    .preceded_by(tag("Program: "))
    .parse(input)
}

#[derive(Debug)]
pub struct Input {
    initial_registers: EnumMap<cpu::Register, usize>,
    program: Vec<cpu::Code>,
}

impl Input {
    fn machine(&self) -> cpu::Machine {
        cpu::Machine::new(self.initial_registers, &self.program)
    }
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    parse_registers
        .terminated(multispace1)
        .and(parse_program)
        .map(|(registers, program)| Input {
            initial_registers: registers,
            program,
        })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

impl Display for cpu::Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use cpu::Code::*;

        let value = match self {
            Zero => "0",
            One => "1",
            Two => "2",
            Three => "3",
            Four => "4",
            Five => "5",
            Six => "6",
            Seven => "7",
        };

        f.write_str(value)
    }
}

pub fn part1(input: Input) -> anyhow::Result<impl Display> {
    let mut machine = input.machine();

    let mut outs = Vec::new();

    while let cpu::MachineState::Output(value) = machine.run_until_state() {
        outs.push(value);
    }

    Ok(outs.join_with(Comma))
}

pub fn part2(input: Input) -> anyhow::Result<usize> {
    let mut candidate = 1 << 46;

    loop {
        let mut machine = input.machine();
        machine.reinit(candidate);

        let outs = machine.run_iter();

        let miss = outs
            .zip_longest(input.program.iter().copied())
            .position(|pair| match pair {
                EitherOrBoth::Both(out, target) => out != target,
                EitherOrBoth::Left(_) => true,
                EitherOrBoth::Right(_) => true,
            });

        match miss {
            None => return Ok(candidate),
            Some(idx) => {
                if idx >= 9 {
                    eprintln!("fixing {idx}");
                }

                // Find the bit that fixes this index, and increment there
                let bit = 1usize << (idx * 3);
                candidate += bit;

                // Zero out the 3 bits to the right of that bit, because they
                // need to be recalculated
                let mask = 0b111 << (idx * 3);
                let mask = mask >> 3;
                candidate &= !mask;
            }
        }
    }
}

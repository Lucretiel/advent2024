use nom::{
    branch::alt,
    character::complete::{digit1, newline, space1},
    combinator::{eof, success},
    Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
    tag::complete::tag, ParserExt,
};

use crate::library::{Definitely, ITResult};

#[derive(Debug)]
struct Equation {
    value: i64,
    operands: Vec<i64>,
}

impl Equation {
    fn valid(&self, allow_concat: bool) -> bool {
        match self.operands.split_last() {
            None => false,
            Some((&tail, list)) => matches(self.value, list, tail, allow_concat),
        }
    }
}

fn parse_number(input: &str) -> ITResult<&str, i64> {
    digit1.parse_from_str_cut().parse(input)
}

fn parse_equation(input: &str) -> ITResult<&str, Equation> {
    parse_number
        .terminated(tag(": "))
        .and(collect_separated_terminated(
            parse_number,
            space1,
            alt((eof.value(()), newline.value(()))),
        ))
        .map(|(value, operands)| Equation { value, operands })
        .parse(input)
}

#[derive(Debug)]
pub struct Input {
    equations: Vec<Equation>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    collect_separated_terminated(parse_equation, success(()), eof)
        .map(|equations| Input { equations })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

fn matches(target: i64, list: &[i64], tail: i64, allow_concat: bool) -> bool {
    let Some((&next, list)) = list.split_last() else {
        return tail == target;
    };

    if tail > target {
        return false;
    } else if matches(target - tail, list, next, allow_concat) {
        return true;
    } else if target % tail == 0 && matches(target / tail, list, next, allow_concat) {
        return true;
    } else if allow_concat {
        match unconcat(target, tail) {
            None => false,
            Some(out) => matches(out, list, next, allow_concat),
        }
    } else {
        false
    }
}

fn count_digits(value: i64) -> u32 {
    match value {
        0 => 1,
        value => value.ilog10() + 1,
    }
}

fn unconcat(target: i64, value: i64) -> Option<i64> {
    let diff = target - value;
    let digits = count_digits(value);
    let factor = 10i64.pow(digits);

    (diff % factor == 0).then(|| diff / factor)
}

fn solve(input: &Input, allow_concat: bool) -> i64 {
    input
        .equations
        .iter()
        .filter(|eq| eq.valid(allow_concat))
        .map(|eq| eq.value)
        .sum()
}

pub fn part1(input: Input) -> Definitely<i64> {
    Ok(solve(&input, false))
}

pub fn part2(input: Input) -> Definitely<i64> {
    Ok(solve(&input, true))
}

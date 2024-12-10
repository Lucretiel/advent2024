use std::convert::Infallible;

use gridly_grids::VecGrid;
use nom::{
    branch::alt,
    character::complete::char,
    combinator::{eof, success},
    error::ParseError,
    Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated, ParserExt,
};

use crate::library::ITResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Height(u8);

impl Height {
    fn is_valid_successor_from(self, origin: Self) -> bool {
        self.0 == origin.0 + 1
    }

    fn from_char(c: char) -> Option<Self> {
        c.to_digit(10).map(|n| n as u8).map(Height)
    }
}

#[derive(Debug)]
pub struct Input {
    raw: VecGrid<Height>,
}

fn parse_height(input: &str) -> ITResult<&str, Height> {
    let mut chars = input.chars();

    Height::from_char(chars.next().ok_or_else(|| {
        nom::Err::Error(ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Digit,
        ))
    })?)
    .ok_or_else(|| {
        nom::Err::Error(ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Digit,
        ))
    })
    .map(|height| (chars.as_str(), height))
}

fn parse_row(input: &str) -> ITResult<&str, Vec<Height>> {
    collect_separated_terminated(
        parse_height,
        success(()),
        alt((eof.value(()), char('\n').value(()))),
    )
    .parse(input)
}

fn parse_rows(input: &str) -> ITResult<&str, Vec<Vec<Height>>> {
    collect_separated_terminated(parse_row, success(()), eof).parse(input)
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    parse_rows.map_res_cut(|rows| VecGrid::new_from_rows(rows))
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

pub fn part1(input: Input) -> anyhow::Result<Infallible> {
    anyhow::bail!("not implemented yet")
}

pub fn part2(input: Input) -> anyhow::Result<Infallible> {
    anyhow::bail!("not implemented yet")
}

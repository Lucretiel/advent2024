use std::collections::HashSet;

use gridly::prelude::*;
use gridly_grids::VecGrid;
use nom::{
    Parser,
    branch::alt,
    character::complete::char,
    combinator::{eof, success},
    error::ParseError,
};
use nom_supreme::{
    ParserExt, error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
};
use thiserror::Error;

use crate::{
    express,
    library::{Definitely, ITResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Height(u8);

impl Height {
    fn is_valid_successor_from(self, origin: Self) -> bool {
        self.0 == origin.0 + 1
    }

    fn is_start(self) -> bool {
        self.0 == 0
    }

    fn is_summit(self) -> bool {
        self.0 == 9
    }

    fn from_char(c: char) -> Option<Self> {
        c.to_digit(10).map(|n| n as u8).map(Height)
    }
}

#[derive(Debug)]
pub struct Input {
    grid: VecGrid<Height>,
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

#[derive(Debug, Error)]
#[error("input didn't have consistent row lengths")]
struct DimensionError;

fn parse_input(input: &str) -> ITResult<&str, Input> {
    parse_rows
        .map_res_cut(|rows| VecGrid::new_from_rows(rows).ok_or(DimensionError))
        .map(|grid| Input { grid })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

/// Given a particular `location` and `height``, explore all of the neighbors
/// of that location. For each neighbor, if it is a valid successor, call
/// `add_summit`; otherwise, recursively call explore on that location.
fn explore<T>(
    grid: &impl Grid<Item = Height>,
    location: Location,
    height: Height,
    state: T,
    add_summit: &impl Fn(T, Location) -> T,
) -> T {
    EACH_DIRECTION
        .iter()
        .map(|&step| location + step)
        .filter_map(|new_location| {
            grid.get(new_location)
                .ok()
                .map(|&new_height| (new_location, new_height))
        })
        .filter(|&(_, new_height)| new_height.is_valid_successor_from(height))
        .fold(state, |state, (location, height)| {
            if height.is_summit() {
                add_summit(state, location)
            } else {
                explore(grid, location, height, state, add_summit)
            }
        })
}

/// Solve the puzzle by iterating each start point, using `explore` to explore
/// those start points, then adding together the outputs from `count_trails`.
/// For each start point, we use `init_trail` to create some state, pass
/// `add_summit` to explore to explore with that state, then use `count_trails`
/// to summarize the exploration results.
fn solve<T>(
    input: &Input,
    init_trail: impl Fn() -> T,
    add_summit: impl Fn(T, Location) -> T,
    count_trails: impl Fn(T) -> usize,
) -> Definitely<usize> {
    Ok(input
        .grid
        .rows()
        .iter()
        .flat_map(|row| row.iter_with_locations())
        .filter(|&(_, &height)| height.is_start())
        .map(move |(location, &height)| {
            count_trails(explore(
                &input.grid,
                location,
                height,
                init_trail(),
                &add_summit,
            ))
        })
        .sum())
}

pub fn part1(input: Input) -> Definitely<usize> {
    solve(
        &input,
        HashSet::new,
        |reachable_summits, location| express!(reachable_summits.insert(location)),
        |reachable_summits| reachable_summits.len(),
    )
}

pub fn part2(input: Input) -> Definitely<usize> {
    solve(
        &input,
        || 0,
        |trail_count, _| trail_count + 1,
        |trail_count| trail_count,
    )
}

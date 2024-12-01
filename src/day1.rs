use nom::{
    character::complete::{digit1, multispace0, multispace1, space1},
    combinator::eof,
    Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated, ParserExt,
};

use crate::{
    library::{counter::Counter, Definitely, ITResult},
    parser,
};

#[derive(Debug, Default)]
pub struct Input {
    left: Vec<i32>,
    right: Vec<i32>,
}

impl Extend<(i32, i32)> for Input {
    fn extend<T: IntoIterator<Item = (i32, i32)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let len = iter.size_hint().0;

        self.left.reserve(len);
        self.right.reserve(len);

        iter.into_iter().for_each(|(left, right)| {
            self.left.push(left);
            self.right.push(right);
        });
    }
}

fn parse_row(input: &str) -> ITResult<&str, (i32, i32)> {
    parser! {
        digit1.parse_from_str_cut() => left,
        space1,
        digit1.parse_from_str_cut() => right;
        (left, right)
    }
    .parse(input)
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    collect_separated_terminated(parse_row, multispace1, multispace0.terminated(eof)).parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

pub fn part1(mut input: Input) -> Definitely<i32> {
    input.left.sort_unstable();
    input.right.sort_unstable();

    Ok(
        Iterator::zip(input.left.iter().copied(), input.right.iter().copied())
            .map(|(left, right)| (left - right).abs())
            .sum(),
    )
}

pub fn part2(input: Input) -> Definitely<usize> {
    let counts: Counter<i32> = input.right.iter().copied().collect();
    Ok(input
        .left
        .iter()
        .copied()
        .map(|i| i as usize * counts.get(&i))
        .sum())
}

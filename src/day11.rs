use std::{collections::HashMap, convert::Infallible};

use nom::{
    character::complete::{digit1, multispace0, space1},
    combinator::eof,
    Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated, ParserExt,
};

use crate::library::{dynamic, ITResult};
use crate::{day7::count_digits, library::Definitely};

#[derive(Debug)]
pub struct Input {
    values: Vec<i64>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    collect_separated_terminated(
        digit1.parse_from_str_cut::<i64>(),
        space1,
        multispace0.terminated(eof),
    )
    .map(|values| Input { values })
    .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

#[derive(Debug, Clone, Copy)]
enum MaybePair {
    One(i64),
    Pair([i64; 2]),
}

fn split(value: i64) -> MaybePair {
    if value == 0 {
        return MaybePair::One(1);
    };

    let digits = count_digits(value);

    if digits % 2 == 0 {
        let half_digits = digits / 2;
        let power = 10u32.pow(half_digits) as i64;

        let left = value / power;
        let right = value % power;

        MaybePair::Pair([left, right])
    } else {
        MaybePair::One(value * 2024)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Goal {
    value: i64,
    depth: i32,
}

struct DynamicSolution;

impl dynamic::Task<Goal, usize, Infallible> for DynamicSolution {
    type State = MaybePair;

    fn solve<'sub>(
        &self,
        &Goal { value, depth }: &Goal,
        subtasker: &'sub impl dynamic::Subtask<Goal, usize>,
        state: &mut Option<Self::State>,
    ) -> Result<usize, dynamic::TaskInterrupt<'sub, Goal, Infallible>> {
        let &mut pair = match state {
            Some(state) => state,
            None if depth == 0 => return Ok(1),
            None => state.insert(split(value)),
        };

        // TODO: finish implementing tail calls in `dynamic.rs`
        Ok(match pair {
            MaybePair::One(value) => *subtasker.solve(Goal {
                value,
                depth: depth - 1,
            })?,
            MaybePair::Pair([first, second]) => {
                let first = *subtasker.solve(Goal {
                    value: first,
                    depth: depth - 1,
                })?;

                let second = *subtasker.solve(Goal {
                    value: second,
                    depth: depth - 1,
                })?;

                first + second
            }
        })
    }
}

fn solve(values: &[i64], depth: i32) -> usize {
    let mut store = HashMap::new();

    values
        .iter()
        .map(
            |&value| match dynamic::execute(Goal { value, depth }, &DynamicSolution, &mut store) {
                Ok(count) => count,
                Err(err) => match err {
                    dynamic::DynamicError::CircularDependency(_) => panic!(
                        "circular dependency shouldn't be possible, \
                        because each goal's subgoals are depth - 1"
                    ),
                },
            },
        )
        .sum()
}

pub fn part1(input: Input) -> Definitely<usize> {
    Ok(solve(&input.values, 25))
}

pub fn part2(input: Input) -> Definitely<usize> {
    Ok(solve(&input.values, 75))
}

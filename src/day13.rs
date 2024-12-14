use std::ops::{Add, Mul};

use nom::{
    Parser,
    character::complete::{char, digit1, multispace0, space0},
    combinator::{eof, success},
    error::ParseError,
};
use nom_supreme::{
    ParserExt, error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
    tag::complete::tag,
};

use crate::{
    library::{Definitely, ITResult},
    parser,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vector {
    x: i64,
    y: i64,
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<i64> for Vector {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

fn coordinate_parser<'i>(id: char, prefix: char) -> impl Parser<&'i str, i64, ErrorTree<&'i str>> {
    char(id)
        .terminated(char(prefix))
        .precedes(digit1)
        .parse_from_str_cut()
}

fn xy_pair_parser<'i>(prefix: char) -> impl Parser<&'i str, Vector, ErrorTree<&'i str>> {
    parser! {
        coordinate_parser('X', prefix) => x,
        tag(", "),
        coordinate_parser('Y', prefix) => y;
        Vector{x, y}
    }
}

fn object_parser<'i, Object, Output, Error>(
    mut tag: impl Parser<&'i str, Object, Error>,
    mut object: impl Parser<&'i str, Output, Error>,
) -> impl Parser<&'i str, Output, Error>
where
    Error: ParseError<&'i str>,
{
    parser! {
        tag,
        char(':'),
        space0,
        object => out;
        out
    }
}

fn button_parser<'i>(id: char) -> impl Parser<&'i str, Vector, ErrorTree<&'i str>> {
    object_parser(tag("Button ").and(char(id)), xy_pair_parser('+'))
}

fn parse_prize(input: &str) -> ITResult<&str, Vector> {
    object_parser(tag("Prize"), xy_pair_parser('=')).parse(input)
}

#[derive(Debug, Clone, Copy)]
struct Buttons {
    a: Vector,
    b: Vector,
}

#[derive(Debug, Clone, Copy)]
struct Machine {
    buttons: Buttons,
    prize: Vector,
}

fn parse_machine(input: &str) -> ITResult<&str, Machine> {
    parser! {
        button_parser('A').terminated(multispace0) => a,
        button_parser('B').terminated(multispace0) => b,
        parse_prize => prize;
        Machine { buttons: Buttons {a, b}, prize }
    }
    .parse(input)
}

#[derive(Debug)]
pub struct Input {
    machines: Vec<Machine>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    collect_separated_terminated(parse_machine.terminated(multispace0), success(()), eof)
        .map(|machines| Input { machines })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

fn solve_with_math(machine: &Machine) -> Option<i64> {
    // Look, I know the algebra, so I asked wolfram alpha to rearrange the
    // terms here to speed it up.

    // This will return nonsense and probably panic if the slopes of the two
    // lines are the same. We'll cross that bridge when we come to it.

    let x = machine.prize.x;
    let y = machine.prize.y;

    let x1 = machine.buttons.a.x;
    let y1 = machine.buttons.a.y;

    let x2 = machine.buttons.b.x;
    let y2 = machine.buttons.b.y;

    let length1 = (x2 * y - x * y2) / (x2 * y1 - x1 * y2);
    let length2 = (x1 * y - x * y1) / (x1 * y2 - x2 * y1);

    // Check that we have an integer solution. God only knows what happens if
    // we overflowed.
    if machine.buttons.a * length1 + machine.buttons.b * length2 != machine.prize {
        return None;
    }

    let cost_a = length1 * 3;
    let cost_b = length2;

    Some(cost_a + cost_b)
}

fn solve(input: &Input, adjustment: i64) -> Definitely<i64> {
    Ok(input
        .machines
        .iter()
        .filter_map(|machine| {
            let machine = Machine {
                buttons: machine.buttons,
                prize: machine.prize
                    + Vector {
                        x: adjustment,
                        y: adjustment,
                    },
            };
            solve_with_math(&machine)
        })
        .sum())
}

pub fn part1(input: Input) -> Definitely<i64> {
    solve(&input, 0)
}

pub fn part2(input: Input) -> Definitely<i64> {
    solve(&input, 10000000000000)
}

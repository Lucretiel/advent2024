use std::{convert::Infallible, iter};

use memchr::memmem;
use nom::{
    character::complete::{char, digit1},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, final_parser::final_parser, tag::complete::tag, ParserExt};

use crate::{
    library::{Definitely, ITResult},
    parser,
};

#[inline]
fn parse_mul(input: &str) -> IResult<&str, (i32, i32), ()> {
    parser! {
        tag("mul("),
        digit1.parse_from_str_cut() => left,
        char(','),
        digit1.parse_from_str_cut() => right,
        char(')');
        (left, right)
    }
    .parse(input)
}

fn consume_mul_at_point(input: &str) -> i32 {
    parse_mul(input).ok().map(|(_, (a, b))| a * b).unwrap_or(0)
}

pub fn part1(input: &str) -> Definitely<i32> {
    Ok(memmem::find_iter(input.as_bytes(), b"mul")
        .map(|i| consume_mul_at_point(&input[i..]))
        .sum())
}

pub fn part2(input: &str) -> Definitely<i32> {
    let regex = regex::Regex::new(r"do\(\)|don't\(\)|mul\([0-9]{1,3},[0-9]{1,3}\)")
        .expect("regex should have valid syntax");

    let (_, sum) = regex
        .find_iter(input)
        .fold((true, 0), |(enabled, sum), item| {
            let s = item.as_str();

            if s.starts_with("do()") {
                (true, sum)
            } else if s.starts_with("don't") {
                (false, sum)
            } else {
                let product = match enabled {
                    false => 0,
                    true => consume_mul_at_point(s),
                };

                (enabled, sum + product)
            }
        });

    Ok(sum)
}

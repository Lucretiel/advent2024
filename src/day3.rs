use std::iter;

use memchr::memmem;
use nom::{
    character::complete::{char, digit1},
    IResult, Parser,
};
use nom_supreme::{tag::complete::tag, ParserExt};

use crate::{library::Definitely, parser};

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

fn memmem_split<'i>(haystack: &'i str, finder: &memmem::Finder<'_>) -> Option<(&'i str, &'i str)> {
    debug_assert!(std::str::from_utf8(finder.needle()).is_ok());
    finder.find(haystack.as_bytes()).map(|point| {
        let left = &haystack[..point];
        let right = &haystack[point + finder.needle().len()..];
        (left, right)
    })
}

pub fn part2(mut input: &str) -> Definitely<i32> {
    const DO: &str = "do()";
    const DONT: &str = "don't()";

    let do_finder = memmem::Finder::new(DO);
    let dont_finder = memmem::Finder::new(DONT);

    // Create an iterator over all of the enabled zones of the input by scanning
    // for a `don't()` tag, then a `do()` tag.
    let enabled_zones = iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }

        let (zone, tail) = match memmem_split(input, &dont_finder) {
            None => (input, ""),
            Some((zone, tail)) => match memmem_split(tail, &do_finder) {
                None => (zone, ""),
                Some((_, tail)) => (zone, tail),
            },
        };

        input = tail;
        Some(zone)
    });

    let mul_finder = memmem::Finder::new("mul");
    Ok(enabled_zones
        .flat_map(|zone| {
            mul_finder
                .find_iter(zone.as_bytes())
                .map(|i| consume_mul_at_point(&zone[i..]))
        })
        .sum())
}

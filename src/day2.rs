use nom::{character::complete::digit1, Parser};
use nom_supreme::{error::ErrorTree, final_parser::final_parser, ParserExt};

use crate::library::{split_parser, Definitely, ITResult, IterExt};

#[derive(Debug, Clone)]
struct Report {
    levels: Vec<i32>,
}

impl Report {
    fn is_safe(&self) -> bool {
        is_safe_rule(self.levels.iter().copied(), |left, right| {
            matches!(right - left, 1..4)
        }) || is_safe_rule(self.levels.iter().copied(), |left, right| {
            matches!(left - right, 1..4)
        })
    }

    fn is_safe_with_damper(&self) -> bool {
        is_safe_with_damper(&self.levels, |left, right| matches!(right - left, 1..4))
            || is_safe_with_damper(&self.levels, |left, right| matches!(left - right, 1..4))
    }
}

fn parse_level(input: &str) -> ITResult<&str, i32> {
    digit1.parse_from_str_cut().parse(input)
}

fn parse_report(input: &str) -> ITResult<&str, Report> {
    split_parser(parse_level, " ")
        .map(|levels| Report { levels })
        .parse(input)
}

#[derive(Debug)]
pub struct Input {
    reports: Vec<Report>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    split_parser(parse_report, "\n")
        .map(|reports| Input { reports })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

fn is_safe_rule(levels: impl Iterator<Item = i32>, rule: impl Fn(i32, i32) -> bool) -> bool {
    levels
        .streaming_windows()
        .all(|[left, right]| rule(left, right))
}

fn is_safe_with_damper(levels: &[i32], rule: impl Copy + Fn(i32, i32) -> bool) -> bool {
    // Scan from the left to find an problematic index
    let Some(left_unsafe_point) = levels
        .array_windows()
        .position(|&[left, right]| !rule(left, right))
    else {
        return true;
    };

    // Scan from the right to find an problematic index
    let right_unsafe_point = levels
        .array_windows()
        .rposition(|&[left, right]| !rule(left, right))
        .expect("There is known to be at leawst one problematic index");

    match right_unsafe_point - left_unsafe_point {
        // There is precisely one problematic pair, so both of its elements are
        // candidates. Test the report if either item in the pair is removed.
        0 => {
            test_report_with_omitted_index(levels, left_unsafe_point, rule)
                || test_report_with_omitted_index(levels, left_unsafe_point + 1, rule)
        }

        // There are two problematic pairs that share an element. That element
        // is the only candidate for removal.
        1 => test_report_with_omitted_index(levels, right_unsafe_point, rule),

        // The problematic points are too far apart. There is known to be at
        // leat two problematic pairs, so the whole report is unsafe.
        _ => false,
    }
}

// Test if the pair of elements *around* the omitted index fulfill the given
// rule. If either of the elements doesn't exist because it's out of bounds,
// we say that the rule is fulfilled.
#[inline]
fn test_report_with_omitted_index(
    levels: &[i32],
    omit: usize,
    rule: impl Fn(i32, i32) -> bool,
) -> bool {
    let Some(left) = omit.checked_sub(1) else {
        return true;
    };
    let right = omit + 1;

    let Some(&left) = levels.get(left) else {
        return true;
    };
    let Some(&right) = levels.get(right) else {
        return true;
    };

    rule(left, right)
}

pub fn part1(input: Input) -> Definitely<usize> {
    Ok(input
        .reports
        .iter()
        .filter(|report| report.is_safe())
        .count())
}

pub fn part2(input: Input) -> Definitely<usize> {
    Ok(input
        .reports
        .iter()
        .filter(|report| report.is_safe_with_damper())
        .count())
}

use std::{cmp::Ordering, collections::BTreeMap, iter::repeat, mem};

use joinery::JoinableIterator;
use lazy_format::lazy_format;
use nom::{
    Parser,
    character::complete::{alpha1, multispace0, multispace1},
    combinator::{eof, success},
};
use nom_supreme::{
    ParserExt, error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
    tag::complete::tag,
};
use regex::Regex;

use crate::{cmp_all, library::ITResult, parser};

#[derive(Debug)]
pub struct Input<'a> {
    fragments: Vec<&'a str>,
    goals: Vec<&'a str>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    parser! {
        collect_separated_terminated(alpha1, tag(", "), multispace1) => fragments,
        collect_separated_terminated(alpha1.terminated(multispace0), success(()), eof) => goals;
        Input{fragments, goals}
    }
    .parse(input)
}

impl<'a> TryFrom<&'a str> for Input<'a> {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

impl Input<'_> {
    fn compile_components(&self) -> Regex {
        // Safety: we know from the parser that only `[a-zA-Z]` are in
        // the fragments

        let alternations = self
            .fragments
            .iter()
            .map(|fragment| lazy_format!("(:?{fragment})"))
            .join_with('|');

        let looped = lazy_format!("(:?{alternations})*");

        let bounded = format!("^(:?{looped})$");

        Regex::new(&bounded).expect("there shouldn't be a problem compiling this")
    }
}

pub fn part1(input: Input) -> anyhow::Result<usize> {
    let regex = input.compile_components();

    Ok(input
        .goals
        .iter()
        .filter(|goal| regex.is_match(goal))
        .count())
}

/// When searching for solutions, we use these keys, which are substrings that
/// sort by length
#[derive(Debug, Clone, Copy)]
struct Key<'a>(&'a str);

impl Ord for Key<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_all! {
            self.0.len(), other.0.len();
            self.0, other.0;
        }
    }
}

impl PartialOrd for Key<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Eq for Key<'_> {}

impl PartialEq for Key<'_> {
    fn eq(&self, other: &Self) -> bool {
        matches!(Ord::cmp(self, other), Ordering::Equal)
    }
}

pub fn part2(input: Input) -> anyhow::Result<u64> {
    let mut counts: BTreeMap<Key<'_>, u64> = input
        .goals
        .iter()
        .copied()
        .map(Key)
        .zip(repeat(1))
        .collect();

    loop {
        let (Key(key), count) = counts
            .pop_last()
            .expect("map always contains something until loop terminates");

        if key == "" {
            return Ok(count);
        }

        for prefix in &input.fragments {
            if let Some(suffix) = key.strip_prefix(prefix) {
                *counts.entry(Key(suffix)).or_default() += count;
            }
        }
    }
}

use std::{
    collections::{HashMap, HashSet},
    mem::swap,
};

use nom::{
    character::complete::{char, digit1},
    Parser,
};
use nom_supreme::{error::ErrorTree, final_parser::final_parser, ParserExt};

use crate::{
    express,
    library::{split_once_parser, split_parser, Definitely, ITResult},
    parser,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PageNumber(u32);

fn parse_page_number(input: &str) -> ITResult<&str, PageNumber> {
    digit1.parse_from_str_cut().map(PageNumber).parse(input)
}

#[derive(Debug, Clone, Copy)]
struct Rule {
    before: PageNumber,
    after: PageNumber,
}

fn parse_rule(input: &str) -> ITResult<&str, Rule> {
    parser! {
        parse_page_number => before,
        char('|'),
        parse_page_number => after;
        Rule{before, after}
    }
    .parse(input)
}

#[derive(Debug, Default, Clone)]
struct PageRules {
    successors: HashSet<PageNumber>,
}

#[derive(Debug, Default, Clone)]
struct RuleSet {
    rules: HashMap<PageNumber, PageRules>,
}

impl RuleSet {
    fn is_acceptable(&self, before: PageNumber, after: PageNumber) -> bool {
        match self.rules.get(&after) {
            None => true,
            Some(rules) => !rules.successors.contains(&before),
        }
    }
}

impl Extend<Rule> for RuleSet {
    fn extend<T: IntoIterator<Item = Rule>>(&mut self, iter: T) {
        iter.into_iter().for_each(|rule| {
            self.rules
                .entry(rule.before)
                .or_default()
                .successors
                .insert(rule.after);
        });
    }
}

fn parse_rule_set(input: &str) -> ITResult<&str, RuleSet> {
    split_parser(parse_rule, "\n").parse(input)
}

#[derive(Debug, Default, Clone)]
struct Update {
    pages: Vec<PageNumber>,
}

impl Update {
    fn is_sorted(&self, rules: &RuleSet) -> bool {
        self.pages
            .iter()
            .enumerate()
            .map(|(index, &page)| (page, &self.pages[index + 1..]))
            .all(|(page, successors)| {
                successors
                    .iter()
                    .all(|&successor| rules.is_acceptable(page, successor))
            })
    }

    fn middle_page(&self) -> Option<PageNumber> {
        self.pages.get(self.pages.len() / 2).copied()
    }

    fn sort_via_rules(&mut self, rules: &RuleSet) {
        sort_via_rules(&mut self.pages, rules);
    }
}

/// This algorithm is guaranteed to terminate and will produce garbage results
/// if the ordering rules are inconsistent.
fn sort_via_rules(mut pages: &mut [PageNumber], rules: &RuleSet) {
    while let Some((page, tail)) = pages.split_first_mut() {
        sort_head_via_rules(page, tail, rules);
        pages = tail;
    }
}

/// Arrange a set of pages such that `page` is ordered before all of the pages
/// in `tail`, by performing swaps. The pages in tail are left in an
/// indeterminate order.
fn sort_head_via_rules(page: &mut PageNumber, tail: &mut [PageNumber], rules: &RuleSet) {
    let Some((next_page, tail)) = tail.split_first_mut() else {
        return;
    };

    if !rules.is_acceptable(*page, *next_page) {
        swap(page, next_page);
        sort_head_via_rules(page, tail, rules);
    } else if let Some(tail_page) = tail
        .iter_mut()
        .find(|tail_page| !rules.is_acceptable(*page, **tail_page))
    {
        swap(page, tail_page);
        swap(tail_page, next_page);
        sort_head_via_rules(page, tail, rules);
    }
}

impl Extend<PageNumber> for Update {
    fn extend<T: IntoIterator<Item = PageNumber>>(&mut self, iter: T) {
        self.pages.extend(iter)
    }
}

fn parse_update(input: &str) -> ITResult<&str, Update> {
    split_parser(parse_page_number, ",").parse(input)
}

fn parse_updates(input: &str) -> ITResult<&str, Vec<Update>> {
    split_parser(parse_update, "\n").parse(input)
}

#[derive(Debug)]
pub struct Input {
    rules: RuleSet,
    updates: Vec<Update>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    split_once_parser(parse_rule_set, "\n\n")
        .and(parse_updates)
        .map(|(rules, updates)| Input { rules, updates })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

pub fn part1(input: Input) -> Definitely<u32> {
    Ok(input
        .updates
        .iter()
        .filter(|update| update.is_sorted(&input.rules))
        .filter_map(|update| update.middle_page())
        .map(|PageNumber(number)| number)
        .sum())
}

pub fn part2(mut input: Input) -> Definitely<u32> {
    let sum = input
        .updates
        .iter_mut()
        .filter(|update| !update.is_sorted(&input.rules))
        .map(|update| express!(update.sort_via_rules(&input.rules)))
        .filter_map(|update| update.middle_page())
        .map(|PageNumber(number)| number)
        .sum();

    Ok(sum)
}

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::Display,
};

use anyhow::Context;
use gridly::prelude::*;
use gridly_grids::VecGrid;
use lazy_format::lazy_format;
use nom::{
    Parser,
    character::complete::{char, digit1, multispace0},
    combinator::{eof, success},
};
use nom_supreme::{
    ParserExt, error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
};
use rayon::prelude::*;

use crate::{library::ITResult, parser};

#[derive(Debug)]
pub struct Input {
    incoming: Vec<Location>,
}

fn parse_location(input: &str) -> ITResult<&str, Location> {
    parser! {
        digit1.parse_from_str_cut().map(Column) => column,
        char(','),
        digit1.parse_from_str_cut().map(Row) => row;
        Location { row, column }
    }
    .parse(input)
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    collect_separated_terminated(parse_location.terminated(multispace0), success(()), eof)
        .map(|incoming| Input { incoming })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Cell {
    #[default]
    Safe,
    Corrupt,
}

struct SearchStep<const ROW: isize, const COL: isize> {
    location: Location,
    distance: isize,
}

impl<const ROW: isize, const COL: isize> SearchStep<ROW, COL> {
    const fn dest() -> Location {
        Location {
            row: Row(ROW),
            column: Column(COL),
        }
    }

    fn cost(&self) -> isize {
        self.distance + (Self::dest() - self.location).manhattan_length()
    }

    fn done(&self) -> bool {
        self.location == Self::dest()
    }
}

impl<const ROW: isize, const COL: isize> Ord for SearchStep<ROW, COL> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&other.cost(), &self.cost())
    }
}

impl<const ROW: isize, const COL: isize> PartialOrd for SearchStep<ROW, COL> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const ROW: isize, const COL: isize> PartialEq for SearchStep<ROW, COL> {
    fn eq(&self, other: &Self) -> bool {
        self.cost() == other.cost()
    }
}

impl<const ROW: isize, const COL: isize> Eq for SearchStep<ROW, COL> {}

pub fn part1(input: Input) -> anyhow::Result<isize> {
    let mut grid = VecGrid::new((Rows(71), Columns(71))).expect("dimensions are fine");

    input.incoming.iter().take(1024).try_for_each(|&cell| {
        grid.set(cell, Cell::Corrupt)
            .ok()
            .context(lazy_format!("cell out of bounds: {cell:?}"))
    })?;

    let mut seen = HashSet::new();

    let mut queue = BinaryHeap::from([SearchStep::<70, 70> {
        location: Location::zero(),
        distance: 0,
    }]);

    while let Some(step) = queue.pop() {
        if step.done() {
            return Ok(step.distance);
        }

        if seen.replace(step.location).is_some() {
            continue;
        }

        for direction in EACH_DIRECTION {
            let new_location = step.location + direction;
            let distance = step.distance + 1;

            match grid.get(new_location) {
                Err(_) => continue,
                Ok(&Cell::Corrupt) => continue,
                Ok(&Cell::Safe) => {
                    queue.push(SearchStep {
                        location: new_location,
                        distance,
                    });
                }
            }
        }
    }

    anyhow::bail!("no path found")
}

struct TimedGridAdapter<'a> {
    dimensions: Vector,
    cells: &'a HashMap<Location, usize>,
    timestamp: usize,
}

impl GridBounds for TimedGridAdapter<'_> {
    fn dimensions(&self) -> Vector {
        self.dimensions
    }

    fn root(&self) -> Location {
        Location::zero()
    }
}

impl Grid for TimedGridAdapter<'_> {
    type Item = Cell;

    unsafe fn get_unchecked(&self, location: Location) -> &Self::Item {
        match self.cells.get(&location) {
            Some(&timestamp) if timestamp <= self.timestamp => &Cell::Corrupt,
            _ => &Cell::Safe,
        }
    }
}

pub fn part2(input: Input) -> anyhow::Result<impl Display> {
    let cells = input
        .incoming
        .iter()
        .enumerate()
        .map(|(i, &location)| (location, i))
        .collect();

    // We know that at least the first 1024 cells are safe
    let timestamp = (1000..input.incoming.len())
        .into_par_iter()
        .find_first(|&i| {
            let grid = TimedGridAdapter {
                dimensions: Rows(71) + Columns(71),
                cells: &cells,
                timestamp: i,
            };

            let mut seen = HashSet::new();

            let mut queue = BinaryHeap::from([SearchStep::<70, 70> {
                location: Location::zero(),
                distance: 0,
            }]);

            while let Some(step) = queue.pop() {
                if step.done() {
                    return false;
                }

                if seen.replace(step.location).is_some() {
                    continue;
                }

                for direction in EACH_DIRECTION {
                    let new_location = step.location + direction;
                    let distance = step.distance + 1;

                    match grid.get(new_location) {
                        Err(_) => continue,
                        Ok(&Cell::Corrupt) => continue,
                        Ok(&Cell::Safe) => {
                            queue.push(SearchStep {
                                location: new_location,
                                distance,
                            });
                        }
                    }
                }
            }

            true
        })
        .context("no blocking location found")?;

    let location = input.incoming[timestamp];

    Ok(lazy_format!(
        "{x},{y}",
        y = location.row.0,
        x = location.column.0
    ))
}

use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
};

use gridly::prelude::*;
use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::library::{ITResult, IterExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell(u8);

#[derive(Debug)]
pub struct Input {
    map: HashMap<Location, Cell>,
}

impl TryFrom<&str> for Input {
    type Error = Infallible;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let map = value
            .lines()
            .map(|line| line.trim().as_bytes())
            .with_rows(Row(0))
            .flat_map(|(row, line)| {
                line.iter()
                    .copied()
                    .map(Cell)
                    .with_columns(Column(0))
                    .map(move |(column, cell)| (row.combine(column), cell))
            })
            .collect();

        Ok(Input { map })
    }
}

fn explore(
    territory: &HashMap<Location, Cell>,
    root: &Location,
    id: Cell,
    explored_territory: &mut HashSet<&Location>,
) -> usize {
    let mut area = 0;
    let mut perimiter = 0;

    explored_territory.insert(root);
    area += 1;

    for direction in EACH_DIRECTION {}
}

pub fn part1(input: Input) -> anyhow::Result<Infallible> {
    let mut unexplored_territory = HashSet::from_iter(input.map.keys());

    let items = unexplored_territory.drain();

    items.unexplored_territory.drain()
}

pub fn part2(input: Input) -> anyhow::Result<Infallible> {
    anyhow::bail!("not implemented yet")
}

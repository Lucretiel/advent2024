use std::{
    collections::{HashMap, HashSet},
    iter::{self, repeat},
    mem,
};

use anyhow::Context;
use gridly::prelude::*;

use crate::library::IterExt;

#[derive(Debug)]
pub struct Input {
    walls: HashSet<Location>,
    start: Location,
    end: Location,
}

impl TryFrom<&str> for Input {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut walls = HashSet::new();
        let mut start = None;
        let mut end = None;

        for (row, line) in value.lines().map(|line| line.trim()).with_rows(Row(0)) {
            for (column, &cell) in line.as_bytes().iter().with_columns(Column(0)) {
                let location = row + column;
                match cell {
                    b'#' => {
                        walls.insert(location);
                    }
                    b'S' => {
                        if start.replace(location).is_some() {
                            anyhow::bail!("multiple start locations")
                        }
                    }
                    b'E' => {
                        if end.replace(location).is_some() {
                            anyhow::bail!("multiple end locations")
                        }
                    }
                    b'.' => continue,
                    cell => anyhow::bail!("unrecognized cell {cell}", cell = cell as char),
                }
            }
        }

        Ok(Input {
            walls,
            start: start.context("no start location")?,
            end: end.context("no end location")?,
        })
    }
}

fn compute_distance_graph_rooted_at(
    walls: &HashSet<Location>,
    start: &Location,
) -> HashMap<Location, isize> {
    let mut distances = HashMap::new();

    let mut to_explore = HashSet::from([*start]);

    for distance in 0.. {
        let current_step = mem::take(&mut to_explore);

        if current_step.is_empty() {
            return distances;
        }

        distances.extend(current_step.iter().copied().zip(repeat(distance)));

        for &current_location in &current_step {
            for step in EACH_DIRECTION {
                let neighbor = current_location + step;
                if !walls.contains(&neighbor) && !distances.contains_key(&neighbor) {
                    to_explore.insert(neighbor);
                }
            }
        }
    }

    panic!("exhausted an infinite iterator")
}

fn cheat_vectors_distance(distance: isize) -> impl Iterator<Item = Vector> {
    EACH_DIRECTION.iter().flat_map(move |&direction| {
        (0..distance)
            .map(move |turn_point| (turn_point, distance - turn_point))
            .map(move |(d1, d2)| direction * d1 + direction.clockwise() * d2)
    })
}

/// Create an iterator of each step in the path, starting at `start`, where each
/// step is 1 distance away.
fn route(
    distance_graph: &HashMap<Location, isize>,
    start: &Location,
) -> impl Iterator<Item = Location> {
    iter::successors(Some(*start), |current_location| {
        let current_distance = distance_graph.get(current_location)?;
        EACH_DIRECTION
            .iter()
            .map(|&direction| *current_location + direction)
            .find(|new_location| {
                distance_graph.get(new_location).copied() == Some(current_distance - 1)
            })
    })
}

fn solve(input: &Input, max_cheat_distance: isize) -> anyhow::Result<usize> {
    let distance_graph = compute_distance_graph_rooted_at(&input.walls, &input.end);

    eprintln!("finished computing distance graph");

    let unique_cheats = route(&distance_graph, &input.start)
        // Create an iterator of each (current_location, candidate_cheat_location)
        .flat_map(|current_location| {
            (2..=max_cheat_distance).flat_map(move |cheat_distance| {
                cheat_vectors_distance(cheat_distance).map(move |cheat_vector| {
                    (
                        cheat_distance,
                        current_location,
                        current_location + cheat_vector,
                    )
                })
            })
        })
        .filter(
            |&(cheat_distance, ref current_location, ref cheat_location)| {
                let Some(current_distance) = distance_graph.get(current_location) else {
                    return false;
                };
                let Some(distance_after_cheat) = distance_graph.get(cheat_location) else {
                    return false;
                };

                (distance_after_cheat + cheat_distance) <= (current_distance - 100)
            },
        )
        .count();

    Ok(unique_cheats)

    // The problem description claims that there's only one route from start to finish.
    // We take that to mean that the whole graph is a tree.
}

pub fn part1(input: Input) -> anyhow::Result<usize> {
    solve(&input, 2)
}

pub fn part2(input: Input) -> anyhow::Result<usize> {
    solve(&input, 20)
}

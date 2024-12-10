use std::collections::HashSet;

use anyhow::{bail, Context};
use gridly::prelude::*;
use gridly_grids::SparseGrid;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::library::IterExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Obstacle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Guard {
    position: Location,
    direction: Direction,
}

#[derive(Debug)]
pub struct Input {
    grid: SparseGrid<Option<Obstacle>>,
    guard: Guard,
}

impl TryFrom<&str> for Input {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut guard_position = None;
        let mut grid = SparseGrid::new((Rows(0), Columns(0)));

        for (row, line) in value.lines().with_rows(Row(0)) {
            for (column, cell) in line.as_bytes().iter().copied().with_columns(Column(0)) {
                let location = row.combine(column);

                match cell {
                    b'.' => continue,
                    b'^' => {
                        if guard_position.replace(location).is_some() {
                            anyhow::bail!("multiple guards found in grid")
                        }
                    }
                    b'#' => {
                        grid.insert(location, Some(Obstacle));
                    }
                    cell => anyhow::bail!("unrecognized cell {:?} as {location:?}", cell as char),
                }
            }
        }

        let guard_position = guard_position.context("no guard was found in the grid")?;

        Ok(Input {
            grid,
            guard: Guard {
                position: guard_position,
                direction: Up,
            },
        })
    }
}

pub fn part1(Input { mut guard, grid }: Input) -> anyhow::Result<usize> {
    let mut seen_places = HashSet::new();

    loop {
        seen_places.insert(guard.position);

        match [Rotation::None, Clockwise, Rotation::Flip, Anticlockwise]
            .into_iter()
            .find_map(|turn| {
                let direction = guard.direction.rotate(turn);
                let destination = guard.position + direction;
                (grid.get(destination).ok().copied() != Some(Some(Obstacle)))
                    .then_some((destination, direction))
            }) {
            None => bail!("No locations near the guard were available"),
            Some((position, direction)) => match grid.location_in_bounds(position) {
                false => break Ok(seen_places.len()),
                true => {
                    guard.position = position;
                    guard.direction = direction;
                }
            },
        }
    }
}

struct ExtraObstacle<G> {
    grid: G,
    location: Location,
}

impl<G: GridBounds> GridBounds for ExtraObstacle<G> {
    fn dimensions(&self) -> Vector {
        self.grid.dimensions()
    }

    fn root(&self) -> Location {
        self.grid.root()
    }
}

impl<G: Grid<Item = Option<Obstacle>>> Grid for ExtraObstacle<G> {
    type Item = Option<Obstacle>;

    unsafe fn get_unchecked(&self, location: Location) -> &Self::Item {
        const OBSTACLE: Option<Obstacle> = Some(Obstacle);

        if location == self.location {
            &OBSTACLE
        } else {
            self.grid.get_unchecked(location)
        }
    }
}

enum Outcome {
    Loop,
    Exit,
}

fn detect_loop(
    grid: &impl Grid<Item = Option<Obstacle>>,
    mut guard: Guard,
) -> anyhow::Result<Outcome> {
    let mut seen_states = HashSet::new();

    loop {
        if seen_states.insert(guard) == false {
            return Ok(Outcome::Loop);
        }

        match [Rotation::None, Clockwise, Rotation::Flip, Anticlockwise]
            .into_iter()
            .find_map(|turn| {
                let direction = guard.direction.rotate(turn);
                let destination = guard.position + direction;
                (grid.get(destination).ok().copied() != Some(Some(Obstacle)))
                    .then_some((destination, direction))
            }) {
            None => bail!("No locations near the guard were available"),
            Some((position, direction)) => match grid.location_in_bounds(position) {
                false => break Ok(Outcome::Exit),
                true => {
                    guard.position = position;
                    guard.direction = direction;
                }
            },
        }
    }
}

pub fn part2(Input { grid, guard }: Input) -> anyhow::Result<i32> {
    // Why pay for all those cores if we're not gonna use 'em
    (0..grid.num_rows().0)
        .into_par_iter()
        .map(Row)
        .flat_map(|row| {
            (0..grid.num_columns().0)
                .into_par_iter()
                .map(Column)
                .map(move |column| Location::new(row, column))
        })
        .map(|location| ExtraObstacle {
            grid: &grid,
            location,
        })
        .map(|grid| detect_loop(&grid, guard))
        .try_fold(
            || 0,
            |count, outcome| {
                outcome.map(|outcome| match outcome {
                    Outcome::Loop => count + 1,
                    Outcome::Exit => count,
                })
            },
        )
        .try_reduce(|| 0, |a, b| Ok(a + b))
}

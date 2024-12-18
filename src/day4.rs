use anyhow::Context;
use gridly::prelude::*;
use gridly_grids::VecGrid;

use crate::library::Definitely;

#[derive(Debug)]
pub struct Input {
    grid: VecGrid<u8>,
}

impl TryFrom<&str> for Input {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        VecGrid::new_from_rows(value.lines().map(|line| line.as_bytes().iter().copied()))
            .context("grid had inconsistent row lengths")
            .map(|grid| Input { grid })
    }
}

pub fn part1(input: Input) -> Definitely<usize> {
    Ok(input
        .grid
        .rows()
        .iter()
        .flat_map(|row| row.iter_with_locations())
        // For each location in the grid, iterate over the 8 directions
        .flat_map(|(location, _cell)| {
            TOUCHING_ADJACENCIES
                .iter()
                .map(move |&direction| (location, direction))
        })
        // For each candidate location and direction, check if the word "XMAS"
        // is present
        .filter(|&(location, direction)| {
            "XMAS".bytes().zip(0isize..).all(|(byte, offset)| {
                input
                    .grid
                    .get(location + (direction * offset))
                    .map(|&cell| cell == byte)
                    .unwrap_or(false)
            })
        })
        .count())
}

/// Test that an "X-MAS" appears in the grid at the given location.
fn test_mas(grid: &impl Grid<Item = u8>, location: Location) -> bool {
    grid.get(location).copied().ok() == Some(b'A')
        && [Left, Right].iter().all(|&offset| {
            let neighbor = Up + offset;

            let above = grid.get(location + neighbor).copied();
            let below = grid.get(location - neighbor).copied();

            matches!((above, below), (Ok(b'M'), Ok(b'S')) | (Ok(b'S'), Ok(b'M')))
        })
}

pub fn part2(input: Input) -> Definitely<usize> {
    Ok(input
        .grid
        .rows()
        .iter()
        .flat_map(|row| row.iter_with_locations())
        .filter(|&(location, _cell)| test_mas(&input.grid, location))
        .count())
}

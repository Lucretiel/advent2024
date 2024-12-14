use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    fmt::Debug,
    ops::Add,
};

use gridly::prelude::*;

use crate::library::direction_map::DirectionMap;
use crate::{
    direction_map,
    library::{Definitely, IterExt},
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct PlotID(u8);

impl Debug for PlotID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = self.0 as char;
        write!(f, "PlotID(b{c:?})")
    }
}

#[derive(Debug)]
pub struct Input {
    map: HashMap<Location, PlotID>,
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
                    .map(PlotID)
                    .with_columns(Column(0))
                    .map(move |(column, id)| (row.combine(column), id))
            })
            .collect();

        Ok(Input { map })
    }
}

#[inline]
#[must_use]
fn is_different_region(
    territory: &HashMap<Location, PlotID>,
    plot: PlotID,
    location: &Location,
) -> bool {
    territory
        .get(location)
        .map(|&neighbor| neighbor != plot)
        .unwrap_or(true)
}

#[derive(Debug)]
struct Region {
    area: i64,
    perimeter: i64,
}

impl Region {
    fn price(&self) -> i64 {
        self.area * self.perimeter
    }

    fn add_border(self) -> Self {
        Self {
            area: self.area,
            perimeter: self.perimeter + 1,
        }
    }
}

impl Add<Self> for Region {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Region {
            area: self.area + rhs.area,
            perimeter: self.perimeter + rhs.perimeter,
        }
    }
}

fn explore(
    territory: &HashMap<Location, PlotID>,
    location: Location,
    plot: PlotID,
    explored_territory: &mut HashSet<Location>,
) -> Region {
    EACH_DIRECTION
        .iter()
        .map(|&direction| location + direction)
        .fold(
            Region {
                area: 1,
                perimeter: 0,
            },
            |region, neighbor| {
                if is_different_region(territory, plot, &neighbor) {
                    region.add_border()
                } else if explored_territory.replace(neighbor).is_none() {
                    region + explore(territory, neighbor, plot, explored_territory)
                } else {
                    region
                }
            },
        )
}

pub fn part1(input: Input) -> Definitely<i64> {
    let mut explored_territory = HashSet::with_capacity(input.map.len());

    Ok(input
        .map
        .iter()
        .filter_map(
            |(&location, &id)| match explored_territory.replace(location) {
                None => Some(explore(&input.map, location, id, &mut explored_territory)),
                Some(_) => None,
            },
        )
        .map(|region| region.price())
        .sum())
}

fn similar(dir1: Direction, dir2: Direction) -> bool {
    dir1.is_horizontal() == dir2.is_horizontal()
}

fn count_matching_fences(
    local_fences: DirectionMap<bool>,
    neighbor_fences: DirectionMap<bool>,
    direction: Direction,
) -> i64 {
    EACH_DIRECTION
        .iter()
        .filter(|&&candidate| !similar(candidate, direction))
        .filter(|&&candidate| local_fences[candidate])
        .filter(|&&candidate| neighbor_fences[candidate])
        .count() as i64
}

fn explore2(
    territory: &HashMap<Location, PlotID>,
    location: Location,
    plot: PlotID,
    counted_fences: &mut HashMap<Location, DirectionMap<bool>>,
) -> Region {
    let this_region = direction_map! {
        direction => {
            let neighbor = location + direction;
            match is_different_region(territory, plot, &neighbor) {
                false => Some(neighbor),
                true => None,
            }
        }
    };

    let borders = direction_map! {
        direction => this_region[direction].is_none()
    };

    counted_fences.insert(location, borders);

    let perimeter = borders.iter().filter(|&(_, &border)| border).count() as i64;

    // Subtract any fences already counted during earlier iterations. Be sure
    // to do this loop before the true recursion; otherwise fences get double
    // counted
    let perimeter = this_region
        .iter()
        .filter_map(|(direction, neighbor)| {
            counted_fences
                .get(neighbor.as_ref()?)
                .map(|&neighbor_fences| (neighbor_fences, direction))
        })
        .fold(perimeter, |perimeter, (neighbor_fences, direction)| {
            perimeter - count_matching_fences(borders, neighbor_fences, direction)
        });

    // Now we just need to recurse
    this_region
        .iter()
        // Filter neighbor locations that are part of this region.
        .filter_map(|(_, &neighbor)| neighbor)
        .fold(
            Region { area: 1, perimeter },
            |region, neighbor| match counted_fences.get(&neighbor) {
                Some(_) => region,
                // The neighbor is part of this region and hasn't been visited
                // yet. Explore it.
                None => region + explore2(territory, neighbor, plot, counted_fences),
            },
        )
}

pub fn part2(input: Input) -> Definitely<i64> {
    let mut explored_territory = HashSet::with_capacity(input.map.len());

    Ok(input
        .map
        .iter()
        .filter_map(|(&location, &id)| {
            if explored_territory.contains(&location) {
                None
            } else {
                let mut fences = HashMap::new();
                let region = explore2(&input.map, location, id, &mut fences);
                explored_territory.extend(fences.keys().copied());
                Some(region)
            }
        })
        .map(|region| region.price())
        .sum())
}

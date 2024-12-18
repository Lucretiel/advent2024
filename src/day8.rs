use std::collections::{HashMap, HashSet};

use gcd::Gcd;
use gridly::prelude::*;
use nom_supreme::error::ErrorTree;

use crate::library::{Definitely, IterExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Frequency(u8);

#[derive(Debug)]
pub struct Input {
    map: HashMap<Frequency, Vec<Location>>,
    bounds: Vector,
}

impl GridBounds for Input {
    fn dimensions(&self) -> Vector {
        self.bounds
    }

    fn root(&self) -> Location {
        Location::zero()
    }
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut map: HashMap<Frequency, Vec<Location>> = HashMap::new();
        let mut bound = Location::zero();

        for (row, line) in value.lines().with_rows(Row(0)) {
            for (column, cell) in line.trim().bytes().with_columns(Column(0)) {
                let location = Location::new(row, column);

                if location > bound {
                    bound = location;
                }

                if cell == b'.' {
                    continue;
                }

                map.entry(Frequency(cell))
                    .or_default()
                    .push(Location::new(row, column));
            }
        }

        Ok(Input {
            map,
            bounds: bound - (Row(-1), Column(-1)),
        })
    }
}

fn count_antinodes<I>(input: &Input, list_antinodes: impl Fn(Location, Location) -> I) -> usize
where
    I: IntoIterator<Item = Location>,
{
    let mut antinodes = HashSet::new();

    for (&_freq, locations) in input.map.iter() {
        for &location1 in locations.iter() {
            for &location2 in locations.iter() {
                if location1 != location2 {
                    antinodes.extend(
                        list_antinodes(location1, location2)
                            .into_iter()
                            .take_while(|location| input.location_in_bounds(location)),
                    );
                }
            }
        }
    }

    antinodes.len()
}

pub fn part1(input: Input) -> Definitely<usize> {
    let count = count_antinodes(&input, |location1, location2| {
        let vector = location2 - location1;
        let antinode = location1 + (vector * 2);
        [antinode]
    });

    Ok(count)
}

fn reduce(vector: Vector) -> Vector {
    let rows = vector.rows.0.unsigned_abs();
    let columns = vector.columns.0.unsigned_abs();

    let gcd = Gcd::gcd(rows, columns) as isize;

    Vector {
        rows: Rows(vector.rows.0 / gcd),
        columns: Columns(vector.columns.0 / gcd),
    }
}

pub fn part2(input: Input) -> Definitely<usize> {
    let count = count_antinodes(&input, |location1, location2| {
        let vector = location2 - location1;
        let vector = reduce(vector);

        (0..)
            .map(move |factor| vector * factor)
            .map(move |vector| location1 + vector)
    });

    Ok(count)
}

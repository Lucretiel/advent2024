use std::cmp::Ordering;
use std::convert::Infallible;

use anyhow::Context;
use enum_map::{Enum, EnumMap};
use nom::Parser;
use nom::character::complete::{char, digit1, multispace0, space0};
use nom::combinator::{eof, success};
use nom_supreme::ParserExt;
use nom_supreme::multi::collect_separated_terminated;
use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::day13::Vector;
use crate::library::counter::Counter;
use crate::library::{Definitely, ITResult};
use crate::parser;

const ROOM_HEIGHT: i64 = 7;
const ROOM_WIDTH: i64 = 11;

const ROOM: Vector = Vector {
    x: ROOM_WIDTH,
    y: ROOM_HEIGHT,
};

fn parse_coord(input: &str) -> ITResult<&str, i64> {
    digit1
        .opt_preceded_by(char('-'))
        .recognize()
        .parse_from_str_cut()
        .parse(input)
}

fn prefixed_vector_parser<'i>(prefix: char) -> impl Parser<&'i str, Vector, ErrorTree<&'i str>> {
    parser! {
        char(prefix),
        char('='),
        parse_coord => x,
        char(','),
        parse_coord => y;
        Vector { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
struct Robot {
    position: Vector,
    velocity: Vector,
}

fn parse_robot(input: &str) -> ITResult<&str, Robot> {
    parser! {
        prefixed_vector_parser('p') => position,
        space0,
        prefixed_vector_parser('v') => velocity;
        Robot { position, velocity }
    }
    .parse(input)
}

#[derive(Debug)]
pub struct Input {
    robots: Vec<Robot>,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    collect_separated_terminated(parse_robot.terminated(multispace0), success(()), eof)
        .map(|robots| Input { robots })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

fn vector_mod(vector: &Vector, modulus: &Vector) -> Vector {
    Vector {
        x: vector.x.rem_euclid(modulus.x),
        y: vector.y.rem_euclid(modulus.y),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum Zone {
    Lower,
    Upper,
}

fn compute_zone(location: i64, width: i64) -> Option<Zone> {
    // Width 5: [0 1] 2 [3 4]
    let midline = width / 2;
    match Ord::cmp(&location, &midline) {
        Ordering::Less => Some(Zone::Lower),
        Ordering::Equal => None,
        Ordering::Greater => Some(Zone::Upper),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
struct Quadrant {
    x: Zone,
    y: Zone,
}

impl Robot {
    fn take_steps(&mut self, steps: i64, room: &Vector) {
        let total_motion = self.velocity * steps;
        let adjusted_motion = vector_mod(&total_motion, room);
        let new_position = self.position + adjusted_motion;
        self.position = vector_mod(&new_position, room);
    }

    fn compute_quadrant(&self) -> Option<Quadrant> {
        let x = compute_zone(self.position.x, ROOM.x)?;
        let y = compute_zone(self.position.y, ROOM.y)?;

        Some(Quadrant { x, y })
    }
}

fn get_int_var(var: &str) -> anyhow::Result<Option<i64>> {
    match std::env::var(var) {
        Err(err) => match err {
            std::env::VarError::NotPresent => Ok(None),
            std::env::VarError::NotUnicode(_) => {
                anyhow::bail!("environment variable wasn't valid UTF-8")
            }
        },
        Ok(height) => height
            .parse()
            .with_context(|| {
                format!("failed to parse environment variable {height:?} as an integer")
            })
            .map(Some),
    }
}

fn get_room() -> anyhow::Result<Vector> {
    let height = get_int_var("DAY_14_ROOM_HEIGHT").context(
        "error getting room height from \
         environment variable DAY_14_ROOM_HEIGHT",
    )?;
    let width = get_int_var("DAY_14_ROOM_WIDTH").context(
        "error getting room width from \
         environment variable DAY_14_ROOM_WIDTH",
    )?;

    const ROOM_HEIGHT: i64 = 7;
    const ROOM_WIDTH: i64 = 11;

    const ROOM: Vector = Vector {
        x: ROOM_WIDTH,
        y: ROOM_HEIGHT,
    };

    Ok(match (height, width) {
        (None, None) => ROOM,
        (Some(height), Some(width)) => Vector {
            x: width,
            y: height,
        },
        (Some(_), None) => anyhow::bail!("room height was given, but room width was omitted"),
        (None, Some(_)) => anyhow::bail!("room width was given, but room height was omitted"),
    })
}

pub fn part1(mut input: Input) -> anyhow::Result<usize> {
    let room = get_room()?;

    let robot_counts: Counter<Quadrant, EnumMap<Quadrant, usize>> = input
        .robots
        .iter_mut()
        .filter_map(|robot| {
            robot.take_steps(100, &room);
            robot.compute_quadrant()
        })
        .collect();

    // This will be wrong if any quadrant is empty, since counters skip those.
    Ok(robot_counts.iter().map(|(_, count)| count.get()).product())
}

pub fn part2(input: Input) -> anyhow::Result<Infallible> {
    anyhow::bail!("not implemented yet")
}

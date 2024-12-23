use std::cmp::Ordering;

use anyhow::Context;
use enum_map::Enum;
use joinery::JoinableIterator;
use joinery::separators::Newline;
use lazy_format::lazy_format;
use nom::Parser;
use nom::character::complete::{char, digit1, multispace0, space0};
use nom::combinator::{eof, success};
use nom_supreme::ParserExt;
use nom_supreme::multi::collect_separated_terminated;
use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::day13::Vector;
use crate::library::ITResult;
use crate::library::counter::EnumCounter;
use crate::parser;

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
        let new_position = self.position + total_motion;
        self.position = vector_mod(&new_position, room);
    }

    fn compute_quadrant(&self, room: &Vector) -> Option<Quadrant> {
        let x = compute_zone(self.position.x, room.x)?;
        let y = compute_zone(self.position.y, room.y)?;

        Some(Quadrant { x, y })
    }
}

fn get_env_int(var: &str) -> anyhow::Result<Option<i64>> {
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

fn get_env_room() -> anyhow::Result<Option<Vector>> {
    let height = get_env_int("DAY_14_ROOM_HEIGHT").context(
        "error getting room height from \
         environment variable DAY_14_ROOM_HEIGHT",
    )?;
    let width = get_env_int("DAY_14_ROOM_WIDTH").context(
        "error getting room width from \
         environment variable DAY_14_ROOM_WIDTH",
    )?;

    Ok(match (height, width) {
        (None, None) => None,
        (Some(height), Some(width)) => Some(Vector {
            x: width,
            y: height,
        }),
        (Some(_), None) => anyhow::bail!("room height was given, but room width was omitted"),
        (None, Some(_)) => anyhow::bail!("room width was given, but room height was omitted"),
    })
}

#[expect(clippy::format_in_format_args)]
#[allow(unused)]
fn print_room(dimensions: &Vector, robots: &[Robot]) {
    let room = (0..dimensions.y)
        .map(move |y| {
            (0..dimensions.x)
                .map(move |x| {
                    let pos = Vector { x, y };
                    let any = robots.iter().any(|robot| robot.position == pos);
                    lazy_format!(
                        if any => "O"
                        else => "."
                    )
                })
                .join_concat()
        })
        .join_with(Newline);

    eprintln!("{}", format!("{}", room));
}

const DEFAULT_ROOM_HEIGHT: i64 = 103;
const DEFAULT_ROOM_WIDTH: i64 = 101;

const DEFAULT_ROOM: Vector = Vector {
    x: DEFAULT_ROOM_WIDTH,
    y: DEFAULT_ROOM_HEIGHT,
};

pub fn part1(mut input: Input) -> anyhow::Result<usize> {
    eprintln!("got {} robots", input.robots.len());
    let room = get_env_room()?.unwrap_or(DEFAULT_ROOM);

    input
        .robots
        .iter_mut()
        .for_each(|robot| robot.take_steps(100, &room));

    let robot_counts: EnumCounter<Quadrant> = input
        .robots
        .iter()
        .filter_map(|robot| robot.compute_quadrant(&room))
        .collect();

    // This will be wrong if any quadrant is empty, since counters skip those.
    Ok(robot_counts.iter().map(|(_, count)| count.get()).product())
}

/// Return a score indicating the chance that this is the image we want. Higher
/// scores are better
fn score_robots(robots: &[Robot]) -> i64 {
    robots
        .iter()
        .flat_map(|robot1| {
            robots.iter().map(|robot2| {
                let dx = robot1.position.x - robot2.position.x;
                let dy = robot1.position.y - robot2.position.y;

                let dx = dx.abs();
                let dy = dy.abs();

                match (dx, dy) {
                    (0, 1) | (1, 0) => 2,
                    (1, 1) => 1,
                    _ => 0,
                }
            })
        })
        .sum()
}

struct Best<T, const COUNT: usize> {
    items: Vec<T>,
}

impl<T: Clone + Ord, const COUNT: usize> Best<T, COUNT> {
    fn new() -> Self {
        Self {
            items: Vec::with_capacity(COUNT),
        }
    }

    fn insert(&mut self, item: &T) {
        if self.items.len() < COUNT {
            self.items.push(item.clone());
        } else {
            let Some(first) = self.items.first_mut() else {
                return;
            };

            if *first < *item {
                *first = item.clone()
            }
        }

        self.items.sort_unstable();
    }

    fn best(&self) -> Option<&T> {
        self.items.first()
    }
}

#[derive(Debug, Default, Clone)]
struct RobotSnapshot {
    score: i64,
    robots: Vec<Robot>,
}

impl RobotSnapshot {
    fn new(robots: Vec<Robot>) -> Self {
        Self {
            score: score_robots(&robots),
            robots,
        }
    }
    fn step(&mut self, room: &Vector) {
        self.robots
            .iter_mut()
            .for_each(|robot| robot.take_steps(1, room));

        self.score = score_robots(&self.robots)
    }
}

impl PartialEq for RobotSnapshot {
    fn eq(&self, other: &Self) -> bool {
        matches!(Ord::cmp(self, other), Ordering::Equal)
    }
}

impl Eq for RobotSnapshot {}

impl PartialOrd for RobotSnapshot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for RobotSnapshot {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.score, &other.score)
    }
}

pub fn part2(input: Input) -> anyhow::Result<i64> {
    let room = get_env_room()?.unwrap_or(DEFAULT_ROOM);

    // Don't want to deal with off-by-one, so just do 1.5x the cycle length
    let cycle_length = (room.x * room.y * 3) / 2;

    let mut best: Best<_, 1> = Best::new();
    let mut robots = RobotSnapshot::new(input.robots);

    for tick in 1..cycle_length {
        robots.step(&room);

        let pair = (robots, tick);
        best.insert(&pair);
        robots = pair.0;
    }

    Ok(best.best().unwrap().1)
}

use std::collections::{HashMap, HashSet};

use gridly::prelude::*;
use gridly_grids::VecGrid;
use nom::{
    Parser,
    branch::alt,
    character::complete::{char, multispace1},
    combinator::{eof, success},
};
use nom_supreme::{
    ParserExt,
    error::ErrorTree,
    final_parser::final_parser,
    multi::{collect_separated_terminated, parse_separated_terminated},
};

use crate::{
    express,
    library::{Definitely, ITResult, IterExt},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
    Box,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

enum AnyCell {
    Cell(Cell),
    Robot,
}

#[derive(Debug)]
struct Map {
    contents: VecGrid<Cell>,
    robot: Location,
}

#[derive(Debug)]
pub struct Input {
    map: Map,
    instructions: Vec<Direction>,
}

fn parse_cell(input: &str) -> ITResult<&str, AnyCell> {
    use self::Cell::*;
    use AnyCell::*;

    alt((
        char('#').value(Cell(Wall)),
        char('.').value(Cell(Empty)),
        char('O').value(Cell(Box)),
        char('@').value(Robot),
    ))
    .parse(input)
}

fn parse_row(input: &str) -> ITResult<&str, Vec<AnyCell>> {
    collect_separated_terminated(parse_cell, success(()), char('\n')).parse(input)
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("there was no robot in the map")]
    NoRobot,

    #[error("the map wasn't a rectangle")]
    BadDimensions,
}

fn parse_map(input: &str) -> ITResult<&str, Map> {
    collect_separated_terminated(parse_row, success(()), char('\n'))
        // Find the robot in the rows
        .map_res_cut(|lines: Vec<Vec<AnyCell>>| {
            lines
                .iter()
                .with_rows(Row(0))
                .flat_map(|(row, line)| {
                    line.iter()
                        .with_columns(Column(0))
                        .map(move |(column, &cell)| (cell, row.combine(column)))
                })
                .find(|(cell, _loc)| matches!(cell, AnyCell::Robot))
                .ok_or(Error::NoRobot)
                .map(|(_cell, robot_location)| (lines, robot_location))
        })
        // Convert the rows to a VecGrid
        .map_res_cut(|(lines, robot_location)| {
            VecGrid::new_from_rows(lines.iter().map(|line| {
                line.iter().map(|&cell| match cell {
                    AnyCell::Cell(cell) => cell,
                    AnyCell::Robot => Cell::Empty,
                })
            }))
            .ok_or(Error::BadDimensions)
            .map(|grid| Map {
                contents: grid,
                robot: robot_location,
            })
        })
        .parse(input)
}

fn parse_instruction(input: &str) -> ITResult<&str, Direction> {
    alt((
        char('^').value(Up),
        char('>').value(Right),
        char('v').value(Down),
        char('<').value(Left),
    ))
    .parse(input)
}

fn parse_instruction_list(input: &str) -> ITResult<&str, Vec<Direction>> {
    parse_separated_terminated(
        parse_instruction.map(Some).or(multispace1.value(None)),
        success(()),
        eof,
        Vec::new,
        |list, instruction| match instruction {
            None => list,
            Some(instruction) => express!(list.push(instruction)),
        },
    )
    .parse(input)
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    parse_map
        .and(parse_instruction_list)
        .map(|(map, instructions)| Input { map, instructions })
        .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

impl Map {
    // Try to move the robot. Return failure if it couldn't move. In any case,
    // return the robot's new location.
    fn try_move_robot(&mut self, direction: Direction) -> Result<Location, Location> {
        let new_location = self.robot + direction;

        match self.contents.get(new_location) {
            // Out of bounds, or hit a wall, so no movement
            Err(_) | Ok(&Cell::Wall) => Err(self.robot),

            // Cell is empty; robot moves
            Ok(&Cell::Empty) => {
                self.robot = new_location;
                Ok(new_location)
            }

            // There's a box; try to move the box
            Ok(&Cell::Box) => {
                // Loop until we find an empty cell to put the box
                let mut box_location = new_location;

                loop {
                    box_location += direction;

                    match self.contents.get(box_location) {
                        // Out of bounds, or hit a wall, so no movement
                        Err(_) | Ok(&Cell::Wall) => break Err(self.robot),

                        // There's a box; it's part of the group being moved,
                        // so skip it
                        Ok(&Cell::Box) => continue,

                        // Cell is empty; move the box
                        Ok(&Cell::Empty) => {
                            let _ = self.contents.set(box_location, Cell::Box);
                            let _ = self.contents.set(new_location, Cell::Empty);
                            self.robot = new_location;
                            break Ok(new_location);
                        }
                    }
                }
            }
        }
    }
}

fn compute_coordinate(location: &Location) -> isize {
    location.row.0 * 100 + location.column.0
}

pub fn part1(
    Input {
        mut map,
        instructions,
    }: Input,
) -> Definitely<isize> {
    for &direction in &instructions {
        let _res = map.try_move_robot(direction);
    }

    Ok(map
        .contents
        .rows()
        .iter()
        .flat_map(|row| row.iter_with_locations())
        .filter(|&(_, &cell)| matches!(cell, Cell::Box))
        .map(|(location, _)| compute_coordinate(&location))
        .sum())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoxHalf {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell2 {
    Empty,
    Wall,
    Box(BoxHalf),
}

struct Map2 {
    contents: VecGrid<Cell2>,
    robot: Location,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Success,
    Fail,
}

/// Attempt to move a bunch of blocks, such that the robot can be at root.
fn attempt_block_moves(map: &mut VecGrid<Cell2>, root: &Location, direction: Direction) -> Outcome {
    let mut upcoming_checks = Vec::new();
    let mut writes = HashMap::new();
    let mut confirmed = HashSet::new();

    upcoming_checks.push(*root);

    while let Some(location) = upcoming_checks.pop() {
        // For each check, we want the `location` to become empty by pushing
        // whatever box might be there in `direction`

        // Ensure we haven't already evaluated this location
        if confirmed.replace(location).is_some() {
            // Fun fact: I added `confirmed` because, without it, there's a
            // risk of exponential growth if you evaluate a large "pyramid" of
            // boxes (since, if you move a pair of boxes upward, any box above
            // them in the center will be evaluated twice). I tested the code
            // without this, just to see, and it's only marginally slower
            // even on my real input. But I'm keeping it anyway out of
            // algorithmic satisfaction.
            continue;
        }

        match map.get(location) {
            // Hit a wall. None of this will succeed; bail immediately.
            Err(_) | Ok(&Cell2::Wall) => return Outcome::Fail,

            // This location is empty, so there are no problems. Continue
            // with checks
            Ok(&Cell2::Empty) => continue,

            Ok(&Cell2::Box(half)) => {
                // Compute the coordinates of the box
                let (left, right) = match half {
                    BoxHalf::Left => (location, location + Right),
                    BoxHalf::Right => (location + Left, location),
                };

                // Insert the desired writes for the new position of the box.
                writes.insert(left + direction, Cell2::Box(BoxHalf::Left));
                writes.insert(right + direction, Cell2::Box(BoxHalf::Right));

                // Replace the current location of the box with emptiness,
                // unless previous iterations are putting something else there
                // instead
                writes.entry(left).or_insert(Cell2::Empty);
                writes.entry(right).or_insert(Cell2::Empty);

                match direction {
                    Up | Down => {
                        upcoming_checks.push(left + direction);
                        upcoming_checks.push(right + direction);
                    }
                    Left => {
                        upcoming_checks.push(left + Left);
                    }
                    Right => {
                        upcoming_checks.push(right + Right);
                    }
                }
            }
        }
    }

    // All checks succeeded. Execute all writes.
    writes.iter().for_each(|(&location, &cell)| {
        map.set(location, cell)
            .expect("Bounds error during block moves")
    });

    Outcome::Success
}

impl Map2 {
    fn step(&mut self, direction: Direction) {
        let new_location = self.robot + direction;

        match attempt_block_moves(&mut self.contents, &new_location, direction) {
            Outcome::Success => self.robot = new_location,
            Outcome::Fail => {}
        }
    }
}

fn convert_map(map: &Map) -> Map2 {
    use BoxHalf::*;
    use Cell2::*;

    let contents = VecGrid::new_from_rows(map.contents.rows().iter().map(|row| {
        row.iter().flat_map(|&cell| match cell {
            Cell::Empty => [Empty, Empty],
            Cell::Wall => [Wall, Wall],
            Cell::Box => [Box(Left), Box(Right)],
        })
    }))
    .expect("Map should be a rectangle, since the original map is a rectangle");

    let robot = Location {
        row: map.robot.row,
        column: Column(map.robot.column.0 * 2),
    };

    Map2 { contents, robot }
}

pub fn part2(input: Input) -> Definitely<isize> {
    let mut map = convert_map(&input.map);

    input
        .instructions
        .iter()
        .for_each(|&direction| map.step(direction));

    Ok(map
        .contents
        .rows()
        .iter()
        .flat_map(|row| row.iter_with_locations())
        .filter(|&(_, &cell)| matches!(cell, Cell2::Box(BoxHalf::Left)))
        .map(|(location, _)| compute_coordinate(&location))
        .sum())
}

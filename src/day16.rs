use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
};

use gridly::prelude::*;

use crate::library::{IterExt, direction_map::DirectionMap};

#[derive(Debug)]
pub struct Input {
    start: Location,
    end: Location,
    walls: HashSet<Location>,
}

impl TryFrom<&str> for Input {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut start = None;
        let mut end = None;
        let mut walls = HashSet::new();

        for (row, line) in value.lines().map(|line| line.trim()).with_rows(Row(0)) {
            for (column, &cell) in line.as_bytes().iter().with_columns(Column(0)) {
                let location = row + column;

                match cell {
                    b'#' => {
                        walls.insert(Location::new(row, column));
                    }
                    b'.' => {}
                    b'S' if start.is_some() => anyhow::bail!("multiple start locations"),
                    b'S' => start = Some(location),
                    b'E' if end.is_some() => anyhow::bail!("multiple end locations"),
                    b'E' => end = Some(location),
                    _ => anyhow::bail!("invalid cell: {:?}", cell as char),
                }
            }
        }

        Ok(Input {
            start: start.ok_or_else(|| anyhow::anyhow!("no start location"))?,
            end: end.ok_or_else(|| anyhow::anyhow!("no end location"))?,
            walls,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    location: Location,
    direction: Direction,
}

#[derive(Debug, Clone)]
struct Frame {
    cost: i64,
    state: State,
    end: Location,
}

impl Frame {
    fn estimate_overall_cost(&self) -> i64 {
        let vector_to_end = self.end - self.state.location;

        let distance_cost = vector_to_end.manhattan_length() as i64;

        // Need to make at least one turn to move vertically
        let turn_cost_1 =
            if self.end.row != self.state.location.row && self.state.direction.is_horizontal() {
                1000
            } else {
                0
            };

        // Need to make at least one turn to move horizontally
        let turn_cost_2 = if self.end.column != self.state.location.column
            && self.state.direction.is_vertical()
        {
            1000
        } else {
            0
        };

        let turnaround_cost = if let Some(direction) = vector_to_end.direction() {
            // If you're pointing exactly the wrong way, then the turn costs
            // from before didn't apply.
            if self.state.direction == direction.reverse() {
                2000
            } else {
                0
            }
        } else {
            0
        };

        self.cost + distance_cost + turn_cost_1 + turn_cost_2 + turnaround_cost
    }
}

impl Ord for Frame {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort frames such that the "larger" frame has a lower cost
        Ord::cmp(
            &other.estimate_overall_cost(),
            &self.estimate_overall_cost(),
        )
    }
}

impl PartialOrd for Frame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Frame {}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

fn solve_maze(walls: &HashSet<Location>, start: Location, end: Location) -> anyhow::Result<i64> {
    let mut seen_places = HashSet::new();

    let mut exploration_stack: BinaryHeap<Frame> = BinaryHeap::from([Frame {
        cost: 0,
        end,
        state: State {
            location: start,
            direction: Right,
        },
    }]);

    while let Some(frame) = exploration_stack.pop() {
        if frame.state.location == end {
            return Ok(frame.cost);
        }

        if walls.contains(&frame.state.location) {
            continue;
        }

        if seen_places.replace(frame.state).is_some() {
            continue;
        }

        for direction in EACH_DIRECTION {
            exploration_stack.push(Frame {
                cost: frame.cost
                    + if direction == frame.state.direction {
                        1
                    } else {
                        1001
                    },
                state: State {
                    location: frame.state.location + direction,
                    direction,
                },
                end,
            });
        }
    }

    anyhow::bail!("no path found")
}

pub fn part1(input: Input) -> anyhow::Result<i64> {
    solve_maze(&input.walls, input.start, input.end)
}

fn count_path_area(
    end: Location,
    start: Location,
    paths: &HashMap<Location, DirectionMap<bool>>,
) -> usize {
    let mut explored = HashSet::from([start, end]);
    let mut unexplored = Vec::from([end]);

    while let Some(location) = unexplored.pop() {
        if let Some(&routes_into) = paths.get(&location) {
            unexplored.extend(
                routes_into
                    .iter()
                    .filter(|(_, enabled)| **enabled)
                    .map(|(direction, _)| location - direction)
                    .filter(|&neighbor| explored.replace(neighbor).is_none()),
            );
        }
    }

    explored.len()
}

fn count_maze_route_area(
    walls: &HashSet<Location>,
    start: Location,
    end: Location,
) -> anyhow::Result<usize> {
    let mut seen_places = HashSet::new();
    let mut valid_paths: HashMap<Location, DirectionMap<bool>> = HashMap::new();
    let mut final_cost = None;

    let mut exploration_stack: BinaryHeap<Frame> = BinaryHeap::from([Frame {
        cost: 0,
        end,
        state: State {
            location: start,
            direction: Right,
        },
    }]);

    while let Some(frame) = exploration_stack.pop() {
        if let Some(final_cost) = final_cost {
            if frame.cost > final_cost {
                return Ok(count_path_area(end, start, &valid_paths));
            }
        }

        if walls.contains(&frame.state.location) {
            continue;
        }

        valid_paths.entry(frame.state.location).or_default()[frame.state.direction] = true;

        if frame.state.location == end {
            final_cost = Some(frame.cost);
            continue;
        }

        if seen_places.replace(frame.state).is_some() {
            continue;
        }

        for direction in EACH_DIRECTION {
            exploration_stack.push(Frame {
                cost: frame.cost
                    + if direction == frame.state.direction {
                        1
                    } else {
                        1001
                    },
                state: State {
                    location: frame.state.location + direction,
                    direction,
                },
                end,
            });
        }
    }

    anyhow::bail!("no path found")
}

pub fn part2(input: Input) -> anyhow::Result<usize> {
    // First, fill in dead ends.
    count_maze_route_area(&input.walls, input.start, input.end)
}

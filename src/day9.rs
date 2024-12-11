use std::collections::VecDeque;

use nom::{
    character::complete::multispace0,
    combinator::{eof, success},
    error::ParseError,
    Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::parse_separated_terminated, ParserExt,
};

use crate::library::{Definitely, ITResult};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Block {
    start: i64,
    end: i64,
}

impl Block {
    fn width(&self) -> i64 {
        self.end - self.start
    }

    fn checksum_with(&self, factor: i64) -> i64 {
        (self.start..self.end).map(|i| i * factor).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FileID(u32);

impl FileID {
    fn next(self) -> FileID {
        FileID(self.0 + 1)
    }
}

#[derive(Debug, Clone, Default)]
struct Memory {
    allocated: VecDeque<(Block, FileID)>,
    free: VecDeque<Block>,
}

impl Memory {
    fn shift_all(&mut self) {
        let Some((mut active_block, mut file_id)) = self.allocated.pop_back() else {
            return;
        };

        while let Some(free_block) = self.free.pop_front() {
            let active_width = active_block.width();

            if free_block.start > active_block.end {
                // Don't shift a block to the right, that's silly
                break;
            }

            let newly_allocated_block = if free_block.width() <= active_width {
                // The free block is too small, remove it from the free set and
                // use all of it
                free_block
            } else {
                // The free block is large enough, split it and use the left part
                let split_point = free_block.start + active_width;

                self.free.push_front(Block {
                    start: split_point,
                    end: free_block.end,
                });

                Block {
                    start: free_block.start,
                    end: split_point,
                }
            };

            let shifted_size = newly_allocated_block.width();

            // Technically not the correct place, but all we care about is that
            // it's not near the back.
            self.allocated.push_front((newly_allocated_block, file_id));

            // We weren't able to allocate everything, so we need to update
            // our existing block and continue shifting it
            if shifted_size < active_block.width() {
                active_block.end -= shifted_size;
            }
            // The block was fully allocated, so get a new one
            else if let Some((new_block, new_id)) = self.allocated.pop_back() {
                active_block = new_block;
                file_id = new_id;
            }
            // There are no more blocks. This probably shouldn't be possible.
            else {
                panic!("Logic error somewhere; allocated blocks are empty");
            }
        }

        // Re-insert the leftover block
        self.allocated.push_back((active_block, file_id));
    }

    fn shift_all_without_fragmentation(&mut self) {
        for (active_block, _) in self.allocated.iter_mut().rev() {
            // Find a place to put it
            if let Some(free_block) = self
                .free
                .iter_mut()
                // Use take_while to stop searching when the free blocks
                // surpass the position of the active block
                .take_while(|candidate_free_block| candidate_free_block.start < active_block.start)
                .find(|candidate_free_block| candidate_free_block.width() >= active_block.width())
            {
                let start = free_block.start;
                let split_point = start + active_block.width();

                active_block.start = free_block.start;
                active_block.end = split_point;
                free_block.start = split_point;

                // One unfortunate aspect of this design is that we end up with
                // a lot of zero-width free blocks. Oh well.
            }
        }
    }

    fn compute_checksum(&self) -> i64 {
        self.allocated
            .iter()
            .map(|&(ref block, FileID(file_id))| block.checksum_with(file_id as i64))
            .sum()
    }
}

fn parse_digit(input: &str) -> ITResult<&str, i64> {
    let mut chars = input.chars();

    let value = chars
        .next()
        .ok_or_else(|| {
            nom::Err::Error(ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Digit,
            ))
        })?
        .to_digit(10)
        .ok_or_else(|| {
            nom::Err::Error(ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Digit,
            ))
        })?;

    let tail = chars.as_str();
    Ok((tail, value.into()))
}

#[derive(Debug)]
pub struct Input {
    memory: Memory,
}

fn parse_input(input: &str) -> ITResult<&str, Input> {
    let (input, initial_width) = parse_digit(input)?;

    parse_separated_terminated(
        parse_digit.and(parse_digit),
        success(()),
        multispace0.terminated(eof),
        // Initialize a new memory with the initial file
        move || {
            let mut memory = Memory::default();
            memory.allocated.push_back((
                Block {
                    start: 0,
                    end: initial_width,
                },
                FileID(0),
            ));

            (memory, FileID(1), initial_width)
        },
        // For each pair, insert a free block and an allocated block for the
        // next file.
        move |(mut memory, file, free_point), (buffer_width, file_width)| {
            let file_start = free_point + buffer_width;
            let file_end = file_start + file_width;

            memory.free.push_back(Block {
                start: free_point,
                end: file_start,
            });

            memory.allocated.push_back((
                Block {
                    start: file_start,
                    end: file_end,
                },
                file,
            ));

            (memory, file.next(), file_end)
        },
    )
    .map(|(memory, _, _)| Input { memory })
    .parse(input)
}

impl TryFrom<&str> for Input {
    type Error = ErrorTree<nom_supreme::final_parser::Location>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        final_parser(parse_input)(value)
    }
}

pub fn part1(mut input: Input) -> Definitely<i64> {
    input.memory.shift_all();
    Ok(input.memory.compute_checksum())
}

pub fn part2(mut input: Input) -> Definitely<i64> {
    input.memory.shift_all_without_fragmentation();
    Ok(input.memory.compute_checksum())
}

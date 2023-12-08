use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::ControlFlow;

#[derive(Debug, Copy, Clone)]
enum Direction {
    L,
    R,
}

#[derive(Debug)]
struct Destinations<'a> {
    l: &'a str,
    r: &'a str,
}

impl<'a> From<&'a str> for Destinations<'a> {
    fn from(value: &'a str) -> Self {
        Destinations {
            l: &value[7..=9],
            r: &value[12..=14],
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'L' => Direction::L,
            'R' => Direction::R,
            _ => Err("invalid direction")?,
        })
    }
}

trait Node {
    fn is_start(&self) -> bool;
    fn is_end(&self) -> bool;
}

impl Node for &str {
    fn is_start(&self) -> bool {
        *self.as_bytes().last().unwrap() == b'A'
    }

    fn is_end(&self) -> bool {
        *self.as_bytes().last().unwrap() == b'Z'
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = stdin().lock().lines().map_while(Result::ok).collect::<Vec<_>>();
    let mut lines = input.iter();
    let directions_list = lines
        .next()
        .ok_or("expected directions line")?
        .chars()
        .map(Direction::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    lines.next();

    let destinations = lines
        .map(|line| {
            let source = &line[0..3];
            let destinations = Destinations::try_from(line.as_str())?;
            Ok::<_, Box<dyn Error>>((source, destinations))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let origins = destinations
        .keys()
        .filter_map(|key| key.is_start().then_some(*key))
        .collect::<Vec<&str>>();

    let steps = origins
        .par_iter()
        .map(|&origin| {
            let mut origin = origin;
            let mut cycle = directions_list.iter().cycle();
            let mut step = 0u64;

            while !origin.is_end() {
                let direction = cycle.next().unwrap();

                let destination = destinations.get(origin).unwrap();
                origin = match direction {
                    Direction::L => &destination.l,
                    Direction::R => &destination.r,
                };

                step += 1;
            }
            step
        })
        .collect::<Vec<_>>()
        .into_iter()
        .fold(1u64, |acc, step| {
            acc * step
                / match (0..).try_fold((acc, step), |(acc, step), _| match acc % step {
                    0 => ControlFlow::Break(step),
                    x => ControlFlow::Continue((step, x)),
                }) {
                    ControlFlow::Continue(_) => unreachable!(),
                    ControlFlow::Break(lcm) => lcm,
                }
        });

    println!("{steps}");

    Ok::<_, Box<dyn Error>>(())
}

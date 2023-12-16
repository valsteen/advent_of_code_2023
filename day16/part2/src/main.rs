use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{stdin, BufRead};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum TileType {
    Empty,
    MirrorLeft,
    MirrorRight,
    HorizontalSplitter,
    VerticalSplitter,
}

fn parse_map<I, E>(lines: I) -> Result<Vec<Vec<TileType>>, Box<dyn Error>>
where
    I: Iterator<Item = Result<String, E>>,
    E: Error + 'static,
{
    lines
        .map(|line| {
            let line = line?;

            Ok(line
                .into_bytes()
                .into_iter()
                .map(TileType::try_from)
                .collect::<Result<Vec<_>, _>>()?)
        })
        .collect::<Result<Vec<_>, _>>()
}

impl From<TileType> for u8 {
    fn from(value: TileType) -> Self {
        match value {
            TileType::Empty => b'.',
            TileType::MirrorLeft => b'/',
            TileType::MirrorRight => b'\\',
            TileType::HorizontalSplitter => b'-',
            TileType::VerticalSplitter => b'|',
        }
    }
}

impl TryFrom<u8> for TileType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'.' => TileType::Empty,
            b'/' => TileType::MirrorLeft,
            b'\\' => TileType::MirrorRight,
            b'|' => TileType::VerticalSplitter,
            b'-' => TileType::HorizontalSplitter,
            _ => Err(format!("unknown tile {}", char::from(value)))?,
        })
    }
}

enum Deflection {
    Single(Direction),
    Double([Direction; 2]),
}
impl TileType {
    fn deflect(self, direction: Direction) -> Deflection {
        match self {
            TileType::Empty => Deflection::Single(direction),
            TileType::MirrorLeft => Deflection::Single(match direction {
                Direction::N => Direction::E,
                Direction::S => Direction::W,
                Direction::W => Direction::S,
                Direction::E => Direction::N,
            }),
            TileType::MirrorRight => Deflection::Single(match direction {
                Direction::N => Direction::W,
                Direction::S => Direction::E,
                Direction::W => Direction::N,
                Direction::E => Direction::S,
            }),
            TileType::HorizontalSplitter => match direction {
                Direction::S | Direction::N => Deflection::Double([Direction::W, Direction::E]),
                Direction::E | Direction::W => Deflection::Single(direction),
            },
            TileType::VerticalSplitter => match direction {
                Direction::W | Direction::E => Deflection::Double([Direction::S, Direction::N]),
                Direction::S | Direction::N => Deflection::Single(direction),
            },
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
enum Direction {
    N,
    S,
    W,
    E,
}

impl Direction {
    fn next(self, x: usize, y: usize) -> Option<(usize, usize)> {
        let (delta_x, delta_y) = match self {
            Direction::N => (0, -1),
            Direction::S => (0, 1),
            Direction::W => (-1, 0),
            Direction::E => (1, 0),
        };
        Some((x.checked_add_signed(delta_x)?, y.checked_add_signed(delta_y)?))
    }
}

fn beam_map(map: &Vec<Vec<TileType>>, x: usize, y: usize, direction: Direction) -> usize {
    let mut beams = Vec::new();

    match map[y][x].deflect(direction) {
        Deflection::Single(direction) => {
            beams.push((direction, (x, y)));
        }
        Deflection::Double(directions) => {
            for direction in directions {
                beams.push((direction, (x, y)));
            }
        }
    }

    let mut energized = HashMap::<(usize, usize), HashSet<Direction>>::new();

    for &(direction, (x, y)) in &beams {
        energized.entry((x, y)).or_default().insert(direction);
    }

    let dimension_x = map[0].len();
    let dimension_y = map.len();

    while let Some((direction, (x, y))) = beams.pop() {
        let Some((next_x, next_y)) = direction.next(x, y) else {
            continue;
        };
        if (0..dimension_y).contains(&next_y) && (0..dimension_x).contains(&next_x) {
            if !energized.entry((next_x, next_y)).or_default().insert(direction) {
                continue;
            }

            match map[next_y][next_x].deflect(direction) {
                Deflection::Single(direction) => beams.push((direction, (next_x, next_y))),
                Deflection::Double(directions) => {
                    for direction in directions {
                        beams.push((direction, (next_x, next_y)));
                    }
                }
            }
        }
    }
    energized.len()
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = parse_map(stdin().lock().lines())?;

    let energized = (0..map.len())
        .map(|y| (0, y, Direction::E))
        .chain((0..map.len()).map(|y| (map[0].len() - 1, y, Direction::W)))
        .chain((0..map[0].len()).map(|x| (x, 0, Direction::S)))
        .chain((0..map[0].len()).map(|x| (x, map.len() - 1, Direction::N)))
        .par_bridge()
        .map(|(x, y, direction)| beam_map(&map, x, y, direction))
        .max()
        .ok_or("could not find max")?;

    println!("{energized}");

    Ok::<_, Box<dyn Error>>(())
}

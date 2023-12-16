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
    fn next(self, x: i64, y: i64) -> (i64, i64) {
        let (delta_x, delta_y) = match self {
            Direction::N => (0, -1),
            Direction::S => (0, 1),
            Direction::W => (-1, 0),
            Direction::E => (1, 0),
        };
        (x + delta_x, y + delta_y)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = parse_map(stdin().lock().lines())?;

    let mut beams = Vec::from([(Direction::E, (0, 0))]);

    match map[0][0].deflect(Direction::E) {
        Deflection::Single(direction) => {
            beams.push((direction, (0, 0)));
        }
        Deflection::Double(directions) => {
            for direction in directions {
                beams.push((direction, (0, 0)));
            }
        }
    }

    let mut energized = HashMap::<(i64, i64), HashSet<Direction>>::new();

    for &(direction, (x, y)) in &beams {
        energized.entry((x, y)).or_default().insert(direction);
    }

    let dimension_x = i64::try_from(map[0].len()).unwrap();
    let dimension_y = i64::try_from(map.len()).unwrap();

    while let Some((direction, (x, y))) = beams.pop() {
        let (next_x, next_y) = direction.next(x, y);
        if (0..dimension_y).contains(&next_y) && (0..dimension_x).contains(&next_x) {
            if !energized.entry((next_x, next_y)).or_default().insert(direction) {
                continue;
            }

            match map[usize::try_from(next_y).unwrap()][usize::try_from(next_x).unwrap()].deflect(direction) {
                Deflection::Single(direction) => beams.push((direction, (next_x, next_y))),
                Deflection::Double(directions) => {
                    for direction in directions {
                        beams.push((direction, (next_x, next_y)));
                    }
                }
            }
        }
    }

    println!("{}", energized.len());

    Ok::<_, Box<dyn Error>>(())
}

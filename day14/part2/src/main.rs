use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::repeat;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum TileType {
    Round,
    Cube,
    Empty,
}

impl TryFrom<u8> for TileType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'#' => TileType::Cube,
            b'O' => TileType::Round,
            b'.' => TileType::Empty,
            _ => Err(format!("invalid tile {value}"))?,
        })
    }
}

impl From<TileType> for u8 {
    fn from(value: TileType) -> Self {
        match value {
            TileType::Round => b'O',
            TileType::Cube => b'#',
            TileType::Empty => b'.',
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    N,
    S,
    E,
    W,
}

type Map = Vec<Vec<TileType>>;
fn tilt_map(map: &Map, direction: Direction) -> Map {
    let mut result = map
        .iter()
        .map(|line| {
            line.iter()
                .map(|c| match c {
                    TileType::Cube => TileType::Cube,
                    TileType::Empty | TileType::Round => TileType::Empty,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>();

    let (scan, movement) = get_tilt_movement((map[0].len(), map.len()), direction);

    for (mut x, mut y) in scan {
        let tile = map[y][x];
        if tile == TileType::Round {
            while let (Some(result_x), Some(result_y)) = (
                apply_delta(x, map[0].len() - 1, movement.0),
                apply_delta(y, map.len() - 1, movement.1),
            ) {
                if result[result_y][result_x] == TileType::Empty {
                    (x, y) = (result_x, result_y);
                    continue;
                }
                break;
            }
        }
        result[y][x] = tile;
    }

    result
}

fn get_tilt_movement(dimensions: (usize, usize), direction: Direction) -> (Vec<(usize, usize)>, (i8, i8)) {
    match direction {
        Direction::N => (
            (0..dimensions.1)
                .flat_map(|y| (0..dimensions.0).map(move |x| (x, y)))
                .collect::<Vec<_>>(),
            (0, -1),
        ),
        Direction::S => (
            (0..dimensions.1)
                .rev()
                .flat_map(|y| (0..dimensions.0).map(move |x| (x, y)))
                .collect::<Vec<_>>(),
            (0, 1),
        ),
        Direction::E => (
            (0..dimensions.0)
                .rev()
                .flat_map(|x| (0..dimensions.1).map(move |y| (x, y)))
                .collect::<Vec<_>>(),
            (1, 0),
        ),
        Direction::W => (
            (0..dimensions.0)
                .flat_map(|x| (0..dimensions.1).map(move |y| (x, y)))
                .collect::<Vec<_>>(),
            (-1, 0),
        ),
    }
}

fn apply_delta(pos: usize, upper_bound: usize, delta: i8) -> Option<usize> {
    let upper_bound = i64::try_from(upper_bound).unwrap();
    let result = i64::try_from(pos).unwrap() + i64::try_from(delta).unwrap();
    (i64::clamp(result, 0, upper_bound) == result).then(|| usize::try_from(result).unwrap())
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

fn calculate_load(map: &Vec<Vec<TileType>>) -> usize {
    (0..map.len())
        .flat_map(|y| {
            let map = &map;
            (0..map[0].len()).filter_map(move |x| (map[y][x] == TileType::Round).then_some(map.len() - y))
        })
        .sum::<usize>()
}

fn cycle(map: &mut Map) -> impl Iterator<Item = Map> + '_ {
    repeat(()).map(move |()| {
        for direction in [Direction::N, Direction::W, Direction::S, Direction::E] {
            *map = tilt_map(map, direction);
        }
        map.clone()
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = parse_map(stdin().lock().lines())?;

    // apply Floyd's Tortoise and Hare algorithm

    let mut tortoise_start_state = map.clone();
    let mut hare_start_state = map.clone();

    let mut tortoise_iterator = cycle(&mut tortoise_start_state);
    let mut hare_iterator = cycle(&mut hare_start_state);

    let mut tortoise_state;
    let mut hare_state;

    loop {
        hare_iterator.next().unwrap();
        hare_state = hare_iterator.next().unwrap();
        tortoise_state = tortoise_iterator.next().unwrap();

        if tortoise_state == hare_state {
            break;
        }
    }

    let mut tortoise_state = map.clone();
    let mut tortoise_start_state = map.clone();
    let mut tortoise_iterator = cycle(&mut tortoise_start_state);

    let mut cycle_start = 0;

    while tortoise_state != hare_state {
        tortoise_state = tortoise_iterator.next().unwrap();
        hare_state = hare_iterator.next().unwrap();
        cycle_start += 1;
    }

    let mut cycle_length = 1;
    hare_state = hare_iterator.next().unwrap();

    while tortoise_state != hare_state {
        hare_state = hare_iterator.next().unwrap();
        cycle_length += 1;
    }

    let mut cycling_map = map.clone();
    let mut map_cycle = cycle(&mut cycling_map);

    for _ in 0..cycle_start {
        map_cycle.next();
    }

    let reduced_length = (1_000_000_000 - cycle_start) % cycle_length;
    let result_map = map_cycle.take(reduced_length).last().unwrap();

    let score = calculate_load(&result_map);

    println!("{score}");

    Ok::<_, Box<dyn Error>>(())
}

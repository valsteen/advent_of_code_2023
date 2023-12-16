use std::error::Error;
use std::io::{stdin, BufRead};

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

type Map = Vec<Vec<TileType>>;
fn tilt_map(map: &Map) -> Map {
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

    for (mut x, mut y) in (0..map.len())
        .flat_map(|y| (0..map[0].len()).map(move |x| (x, y)))
        .collect::<Vec<_>>()
    {
        let tile = map[y][x];
        if tile == TileType::Round {
            while let (Some(result_x), Some(result_y)) =
                (apply_delta(x, map[0].len() - 1, 0), apply_delta(y, map.len() - 1, -1))
            {
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

fn main() -> Result<(), Box<dyn Error>> {
    let map = parse_map(stdin().lock().lines())?;
    let map = tilt_map(&map);

    let sum = (0..map.len())
        .flat_map(|y| {
            let map = &map;
            (0..map[0].len()).filter_map(move |x| (map[y][x] == TileType::Round).then_some(map.len() - y))
        })
        .sum::<usize>();

    println!("{sum}");

    Ok::<_, Box<dyn Error>>(())
}

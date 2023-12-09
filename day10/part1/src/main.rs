use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::ControlFlow;

#[derive(Debug)]
enum TileType {
    NE,
    NS,
    NW,
    EW,
    ES,
    SW,
}

impl TileType {
    fn neighbours(&self, x: i64, y: i64) -> [(i64, i64); 2] {
        match self {
            TileType::NE => [(x, y - 1), (x + 1, y)],
            TileType::NS => [(x, y - 1), (x, y + 1)],
            TileType::NW => [(x, y - 1), (x - 1, y)],
            TileType::EW => [(x + 1, y), (x - 1, y)],
            TileType::ES => [(x + 1, y), (x, y + 1)],
            TileType::SW => [(x, y + 1), (x - 1, y)],
        }
    }
}

#[derive(Debug)]
enum TileState {
    Known(TileType),
    Unknown,
}

enum ParsedTile {
    TileState(TileState),
    Ground,
    Start,
}

impl TryFrom<char> for ParsedTile {
    type Error = Box<dyn Error>;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(ParsedTile::TileState(TileState::Known(match value {
            '|' => TileType::NS,
            '-' => TileType::EW,
            'L' => TileType::NE,
            'J' => TileType::NW,
            '7' => TileType::SW,
            'F' => TileType::ES,
            '.' => return Ok(ParsedTile::Ground),
            'S' => return Ok(ParsedTile::Start),
            c => Err(format!("invalid character: {c}"))?,
        })))
    }
}

#[derive(Debug)]
struct Map {
    tiles: HashMap<(i64, i64), TileState>,
    start: (i64, i64),
}

impl Map {
    fn try_from<T, S, E>(iterator: T) -> Result<Map, Box<dyn Error>>
    where
        T: Iterator<Item = Result<S, E>>,
        E: Error + 'static,
        S: AsRef<str>,
    {
        let (start, tiles) =
            iterator
                .enumerate()
                .try_fold((None, HashMap::new()), |(start, mut tiles), (y, line)| {
                    let y = i64::try_from(y)?;
                    let line = line?;
                    let chars = line.as_ref().chars();
                    let reserve = chars.size_hint();
                    tiles.reserve(reserve.1.unwrap_or(reserve.0));
                    let (start, tiles) = chars.map(ParsedTile::try_from).enumerate().try_fold(
                        (start, tiles),
                        |(mut start, mut tiles), (x, parsed_tile)| {
                            let x = i64::try_from(x)?;
                            match parsed_tile {
                                Ok(ParsedTile::Start) => {
                                    match start {
                                        None => start = Some((x, y)),
                                        Some((start_x, start_y)) => Err(format!(
                                            "start already given at ({start_x} {start_y}), another start at ({x} {y})"
                                        ))?,
                                    };
                                    tiles.insert((x, y), TileState::Unknown);
                                }
                                Ok(ParsedTile::TileState(tile_state)) => {
                                    tiles.insert((x, y), tile_state);
                                }
                                Ok(ParsedTile::Ground) => (),
                                Err(e) => Err(e)?,
                            };

                            Ok::<_, Box<dyn Error>>((start, tiles))
                        },
                    )?;
                    Ok::<_, Box<dyn Error>>((start, tiles))
                })?;
        Ok(Map {
            tiles,
            start: start.ok_or("no starting point was found")?,
        })
    }

    fn next_from(&self, x: i64, y: i64, exclude_x: i64, exclude_y: i64) -> Result<(i64, i64), Box<dyn Error>> {
        Ok(self
            .all_next_from(x, y)?
            .into_iter()
            .find(|&(dest_x, dest_y)| (dest_x, dest_y) != (exclude_x, exclude_y))
            .unwrap())
    }

    fn all_next_from(&self, x: i64, y: i64) -> Result<[(i64, i64); 2], Box<dyn Error>> {
        match self.tiles.get(&(x, y)).ok_or("there is no tile at this position")? {
            TileState::Known(tile_type) => Ok(tile_type.neighbours(x, y)),
            TileState::Unknown => {
                let mut possible_neighbours = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
                    .into_iter()
                    .filter_map(|(delta_x, delta_y)| {
                        self.all_next_from(x + delta_x, y + delta_y)
                            .into_iter()
                            .flatten()
                            .any(|(dest_x, dest_y)| (dest_x, dest_y) == (x, y))
                            .then_some((x + delta_x, y + delta_y))
                    });

                let result = [
                    possible_neighbours.next().ok_or("no possible neighbour was found")?,
                    possible_neighbours.next().ok_or("second neighbour was not found")?,
                ];
                if possible_neighbours.next().is_some() {
                    Err("more than 2 possible neighbours returned")?;
                }
                Ok(result)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = Map::try_from(stdin().lock().lines())?;

    let [pos_1, pos_2] = map.all_next_from(map.start.0, map.start.1)?;

    let steps = match (2..).try_fold(
        (map.start, pos_1, map.start, pos_2),
        |(prev_1, pos_1, prev_2, pos_2), i| match (
            map.next_from(pos_1.0, pos_1.1, prev_1.0, prev_1.1),
            map.next_from(pos_2.0, pos_2.1, prev_2.0, prev_2.1),
        ) {
            (Ok(next_1), Ok(next_2)) => {
                if next_1 == next_2 {
                    ControlFlow::Break(Ok(i))
                } else {
                    ControlFlow::Continue((pos_1, next_1, pos_2, next_2))
                }
            }
            (Err(e), _) | (_, Err(e)) => ControlFlow::Break(Err::<_, Box<dyn Error>>(e)),
        },
    ) {
        ControlFlow::Continue(_) => unreachable!(),
        ControlFlow::Break(Err(e)) => Err(e)?,
        ControlFlow::Break(Ok(steps)) => steps,
    };
    println!("{steps}");
    Ok::<_, Box<dyn Error>>(())
}

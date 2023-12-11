use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::repeat;
use std::ops::ControlFlow;

#[derive(Debug, Copy, Clone)]
enum TileType {
    NE,
    NS,
    NW,
    EW,
    ES,
    SW,
}

impl TileType {
    fn all_types() -> [TileType; 6] {
        [
            TileType::NE,
            TileType::NS,
            TileType::NW,
            TileType::EW,
            TileType::ES,
            TileType::SW,
        ]
    }
    fn zoom(self) -> [[bool; 3]; 3] {
        let o = true;
        let x = false;

        match self {
            TileType::NE => [
                [x, o, x], //
                [x, o, o], //
                [x, x, x], //
            ], //
            TileType::NS => [
                [x, o, x], //
                [x, o, x], //
                [x, o, x], //
            ], //
            TileType::NW => [
                [x, o, x], //
                [o, o, x], //
                [x, x, x], //
            ], //
            TileType::EW => [
                [x, x, x], //
                [o, o, o], //
                [x, x, x], //
            ], //
            TileType::ES => [
                [x, x, x], //
                [x, o, o], //
                [x, o, x], //
            ], //
            TileType::SW => [
                [x, x, x], //
                [o, o, x], //
                [x, o, x], //
            ], //
        }
    }

    fn neighbours(self, x: i64, y: i64) -> [(i64, i64); 2] {
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

#[derive(Debug, Copy, Clone)]
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
            .all_next_from(*self.tiles.get(&(x, y)).ok_or("no tile at this position")?, x, y)?
            .into_iter()
            .find(|&(dest_x, dest_y)| (dest_x, dest_y) != (exclude_x, exclude_y))
            .unwrap())
    }

    fn all_next_from(&self, tile_state: TileState, x: i64, y: i64) -> Result<[(i64, i64); 2], Box<dyn Error>> {
        match tile_state {
            TileState::Known(tile_type) => Ok(tile_type.neighbours(x, y)),
            TileState::Unknown => {
                let mut possible_neighbours = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
                    .into_iter()
                    .filter_map(|(delta_x, delta_y)| {
                        self.all_next_from(*self.tiles.get(&(x + delta_x, y + delta_y))?, x + delta_x, y + delta_y)
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

trait PaintMapTrait {
    fn paint(&mut self, x: i64, y: i64, tile_type: TileType);

    fn paint_circuit(
        &mut self,
        map: &Map,
        start_positions: [((i64, i64), (i64, i64)); 2],
    ) -> Result<PaintMapResult, Box<dyn Error>>;
}

type PaintMap = HashSet<(i64, i64)>;
impl PaintMapTrait for PaintMap {
    fn paint(&mut self, x: i64, y: i64, tile_type: TileType) {
        for (delta_y, row) in tile_type.zoom().into_iter().enumerate() {
            for (delta_x, occupied) in row.into_iter().enumerate() {
                let delta_x = i64::try_from(delta_x).unwrap();
                let delta_y = i64::try_from(delta_y).unwrap();
                let (x, y) = (x * 3 + delta_x, y * 3 + delta_y);
                if occupied {
                    self.insert((x, y));
                }
            }
        }
    }

    fn paint_circuit(
        &mut self,
        map: &Map,
        start_positions: [((i64, i64), (i64, i64)); 2],
    ) -> Result<PaintMapResult, Box<dyn Error>> {
        let (min_x, min_y, max_x, max_y) = match repeat(()).try_fold(
            (i64::MAX, i64::MAX, i64::MIN, i64::MIN, start_positions),
            |(mut min_x, mut min_y, mut max_x, mut max_y, start_positions), ()| {
                for (x, y) in [start_positions[0].1, start_positions[1].1] {
                    match map.tiles.get(&(x, y)) {
                        Some(TileState::Unknown) | None => unreachable!(),
                        Some(&TileState::Known(tile_type)) => {
                            self.paint(x, y, tile_type);

                            for delta_y in 0..3 {
                                for delta_x in 0..3 {
                                    let (x, y) = (x * 3 + delta_x, y * 3 + delta_y);
                                    (min_x, min_y, max_x, max_y) =
                                        (x.min(min_x), y.min(min_y), x.max(max_x), y.max(max_y));
                                }
                            }
                        }
                    }
                }
                if start_positions[0].1 == start_positions[1].1 {
                    ControlFlow::Break(Ok((min_x, min_y, max_x, max_y)))
                } else {
                    match (
                        map.next_from(
                            start_positions[0].1 .0,
                            start_positions[0].1 .1,
                            start_positions[0].0 .0,
                            start_positions[0].0 .1,
                        ),
                        map.next_from(
                            start_positions[1].1 .0,
                            start_positions[1].1 .1,
                            start_positions[1].0 .0,
                            start_positions[1].0 .1,
                        ),
                    ) {
                        (Ok(next_1), Ok(next_2)) => ControlFlow::Continue((
                            min_x,
                            min_y,
                            max_x,
                            max_y,
                            [(start_positions[0].1, next_1), (start_positions[1].1, next_2)],
                        )),
                        (Err(e), _) | (_, Err(e)) => ControlFlow::Break(Err::<_, Box<dyn Error>>(e)),
                    }
                }
            },
        ) {
            ControlFlow::Continue(_) => unreachable!(),
            ControlFlow::Break(Err(e)) => Err(e)?,
            ControlFlow::Break(Ok(steps)) => steps,
        };
        Ok(PaintMapResult {
            min: (min_x, min_y),
            max: (max_x, max_y),
        })
    }
}

struct PaintMapResult {
    min: (i64, i64),
    max: (i64, i64),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut map = Map::try_from(stdin().lock().lines())?;

    let [start_neighbour_1, start_neighbour_2] = map.all_next_from(TileState::Unknown, map.start.0, map.start.1)?;

    let guessed_type = TileType::all_types()
        .into_iter()
        .find(|&tile_type| {
            map.all_next_from(TileState::Known(tile_type), map.start.0, map.start.1)
                .map(|found| {
                    found
                        .into_iter()
                        .all(|pos| [start_neighbour_1, start_neighbour_2].contains(&pos))
                })
                .unwrap_or_default()
        })
        .ok_or("cannot guess starting tile type")?;

    let mut painted_map = HashSet::new();
    painted_map.paint(map.start.0, map.start.1, guessed_type);

    map.tiles
        .insert((map.start.0, map.start.1), TileState::Known(guessed_type));

    let PaintMapResult {
        min: (min_x, min_y),
        max: (max_x, max_y),
    } = painted_map.paint_circuit(&map, [(map.start, start_neighbour_1), (map.start, start_neighbour_2)])?;

    let mut queue = Vec::from([(min_x, min_y)]);
    painted_map.insert((min_x, min_y));

    while let Some((x, y)) = queue.pop() {
        for (delta_x, delta_y) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
            let x = x + delta_x;
            let y = y + delta_y;
            if !(min_x..=max_x).contains(&x) || !(min_y..=max_y).contains(&y) {
                continue;
            }
            if painted_map.insert((x, y)) {
                queue.push((x, y));
            }
        }
    }

    let sum = (min_y..max_y)
        .step_by(3)
        .map(|y| {
            (min_x..max_x)
                .step_by(3)
                .filter(|x| {
                    (0..3).all(|delta_y| {
                        (0..3).all(|delta_x| {
                            let x = x + delta_x;
                            let y = y + delta_y;
                            !painted_map.contains(&(x, y))
                        })
                    })
                })
                .count()
        })
        .sum::<usize>();

    println!("{sum}");
    Ok::<_, Box<dyn Error>>(())
}

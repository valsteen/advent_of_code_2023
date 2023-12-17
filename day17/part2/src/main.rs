use atomic::AtomicI64;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::sync::atomic;

fn parse_map<I, E>(lines: I) -> Result<Vec<Vec<i64>>, Box<dyn Error>>
where
    I: Iterator<Item = Result<String, E>>,
    E: Error + 'static,
{
    lines
        .map(|line| {
            let line = line?;

            line.chars()
                .map(|c| {
                    Ok::<_, Box<dyn Error>>(c.to_digit(10).ok_or_else(|| format!("invalid digit {c}"))?.try_into()?)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum Direction {
    N,
    E,
    W,
    S,
}

#[derive(Debug)]
struct State {
    x: i64,
    y: i64,
    remaining_straight_moves: u8,
    can_turn: bool,
    direction: Direction,
    heat_loss: i64,
}

impl Clone for State {
    fn clone(&self) -> Self {
        State {
            x: self.x,
            y: self.y,
            remaining_straight_moves: self.remaining_straight_moves,
            can_turn: self.can_turn,
            direction: self.direction,
            heat_loss: self.heat_loss,
        }
    }
}

impl State {
    fn next<'a>(&'a self, map: &'a Vec<Vec<i64>>) -> impl Iterator<Item = State> + '_ {
        [
            {
                (self.remaining_straight_moves > 0).then(|| {
                    let (x, y) = self.direction.next(self.x, self.y);
                    (
                        true,
                        State {
                            x,
                            y,
                            remaining_straight_moves: self.remaining_straight_moves - 1,
                            direction: self.direction,
                            can_turn: self.remaining_straight_moves <= 7,
                            heat_loss: self.heat_loss,
                        },
                    )
                })
            },
            {
                (self.can_turn && !(7..=9).contains(&self.remaining_straight_moves)).then(|| {
                    (
                        false,
                        State {
                            x: self.x,
                            y: self.y,
                            remaining_straight_moves: 10,
                            direction: self.direction.left(),
                            can_turn: false,
                            heat_loss: self.heat_loss,
                        },
                    )
                })
            },
            {
                (self.can_turn && !(7..=9).contains(&self.remaining_straight_moves)).then(|| {
                    (
                        false,
                        State {
                            x: self.x,
                            y: self.y,
                            remaining_straight_moves: 10,
                            direction: self.direction.right(),
                            can_turn: false,
                            heat_loss: self.heat_loss,
                        },
                    )
                })
            },
        ]
        .into_iter()
        .filter_map(|s| {
            s.filter(|(_, s)| {
                (0..i64::try_from(map.len()).unwrap()).contains(&s.y)
                    && (0..i64::try_from(map[0].len()).unwrap()).contains(&s.x)
            })
            .map(|(consume, mut state)| {
                if consume {
                    state.heat_loss += map[usize::try_from(state.y).unwrap()][usize::try_from(state.x).unwrap()];
                }
                state
            })
        })
    }
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

    fn left(self) -> Self {
        match self {
            Direction::N => Direction::W,
            Direction::S => Direction::E,
            Direction::W => Direction::S,
            Direction::E => Direction::N,
        }
    }

    fn right(self) -> Self {
        match self {
            Direction::N => Direction::E,
            Direction::S => Direction::W,
            Direction::W => Direction::N,
            Direction::E => Direction::S,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = parse_map(stdin().lock().lines())?;

    let mut queue = Vec::from([
        State {
            x: 0,
            y: 0,
            remaining_straight_moves: 10,
            can_turn: true,
            direction: Direction::E,
            heat_loss: 0,
        },
        State {
            x: 0,
            y: 0,
            remaining_straight_moves: 10,
            can_turn: true,
            direction: Direction::S,
            heat_loss: 0,
        },
    ]);

    let min: AtomicI64 = AtomicI64::new(i64::MAX);

    let map = &map;

    let best_at: HashMap<(i64, i64, u8, Direction, bool), AtomicI64> = (0..map.len())
        .flat_map(move |y| (0..map[0].len()).map(move |x| (i64::try_from(x).unwrap(), i64::try_from(y).unwrap())))
        .flat_map(|(x, y)| (0..=10).map(move |remaining| (x, y, remaining)))
        .flat_map(|(x, y, remaining)| {
            [true, false]
                .into_iter()
                .map(move |can_turn| (x, y, remaining, can_turn))
        })
        .flat_map(|(x, y, remaining, can_turn)| {
            [Direction::S, Direction::E, Direction::W, Direction::N]
                .into_iter()
                .map(move |direction| (x, y, remaining, direction, can_turn))
        })
        .map(|key| (key, AtomicI64::new(i64::MAX)))
        .collect();

    loop {
        let next = queue
            .drain(..)
            .par_bridge()
            .map(|state| {
                let mut queue = Vec::from([state]);

                while let Some(state) = queue.pop() {
                    if usize::try_from(state.x).unwrap() == map[0].len() - 1
                        && usize::try_from(state.y).unwrap() == map.len() - 1
                        && !(7..=9).contains(&state.remaining_straight_moves)
                    {
                        let current = min.fetch_min(state.heat_loss, atomic::Ordering::Relaxed);
                        if state.heat_loss > current {
                            continue;
                        }
                    } else if state.heat_loss >= min.load(atomic::Ordering::Relaxed) {
                        continue;
                    }

                    let best = best_at
                        .get(&(
                            state.x,
                            state.y,
                            state.remaining_straight_moves,
                            state.direction,
                            state.can_turn,
                        ))
                        .unwrap();

                    let min = best.fetch_min(state.heat_loss, atomic::Ordering::Relaxed);

                    if min <= state.heat_loss {
                        continue;
                    }

                    queue.extend(state.next(map));
                    if queue.len() > 50 {
                        return Some(queue);
                    }
                }
                None::<Vec<State>>
            })
            .flatten()
            .flatten()
            .collect::<Vec<_>>();
        queue.extend(next);
        if queue.is_empty() {
            break;
        }
    }

    println!("{}", min.load(atomic::Ordering::Relaxed));

    Ok::<_, Box<dyn Error>>(())
}

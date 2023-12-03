use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{stdin, BufRead};

struct Number {
    number: u32,
    x: usize,
    y: usize,
}

impl Number {
    fn neighbours(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let x: i32 = self.x.try_into().unwrap();
        let y: i32 = self.y.try_into().unwrap();
        let len: i32 = (self.number.ilog10() as usize + 1).try_into().unwrap();

        let top = ((x - 1)..=(x + len)).map(move |x| (x, y - 1));
        let bottom = ((x - 1)..=(x + len)).map(move |x| (x, y + 1));
        let mid = [(x - 1, y), (x + len, y)];

        top.chain(mid)
            .chain(bottom)
            .filter(|&(x, y)| x >= 0 && y >= 0)
            .map(|(x, y)| (x.try_into().unwrap(), y.try_into().unwrap()))
    }
}

fn main() {
    let lines = stdin().lock().lines();
    let mut symbols = HashMap::new();

    let numbers: Vec<_> = lines
        .flatten()
        .enumerate()
        .flat_map(|(y, line)| {
            for (x, c) in line.as_bytes().iter().enumerate() {
                if !c.is_ascii_digit() && *c != b'.' {
                    symbols.insert((x, y), Vec::with_capacity(2));
                }
            }

            let mut x = 0;
            line.split_terminator(|c: char| !c.is_ascii_digit())
                .filter_map(|l| {
                    let result = match l.parse::<u32>() {
                        Ok(number) => Some(Number { number, x, y }),
                        Err(_) => None,
                    };
                    x += l.len() + 1;
                    result
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for number in numbers {
        for (x, y) in number.neighbours() {
            let pos = (x, y);
            if let Entry::Occupied(mut symbol) = symbols.entry(pos) {
                let numbers = symbol.get_mut();
                if numbers.len() == 2 {
                    symbol.remove();
                } else {
                    numbers.push(number.number);
                }
            }
        }
    }

    let sum: u32 = symbols.values().fold(0, |acc, numbers| {
        acc + (numbers.len() == 2)
            .then(|| numbers[0] * numbers[1])
            .unwrap_or_default()
    });

    println!("{sum}");
}

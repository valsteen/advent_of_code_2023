use std::collections::HashSet;
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
    let mut symbols = HashSet::new();

    let numbers: Vec<_> = lines
        .flatten()
        .enumerate()
        .flat_map(|(y, line)| {
            for (x, c) in line.as_bytes().iter().enumerate() {
                if !c.is_ascii_digit() && *c != b'.' {
                    symbols.insert((x, y));
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
    let sum = numbers.iter().fold(0, |acc, number| {
        acc + number
            .neighbours()
            .any(|(x, y)| symbols.contains(&(x, y)))
            .then_some(number.number)
            .unwrap_or_default()
    });
    println!("{sum}");
}

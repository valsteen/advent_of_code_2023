use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};

#[derive(Hash, Eq, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let sum = lines.map_while(Result::ok).try_fold(0, |acc, line| {
        let (_, line) = line.split_once(": ").ok_or_else(|| format!("Malformed line: {line}"))?;
        let mut maxes = HashMap::<_, u64>::new();
        for hand in line.split("; ") {
            for n_cube in hand.split(", ") {
                let (n, cube) = n_cube
                    .split_once(' ')
                    .ok_or_else(|| format!("Malformed cube information: {n_cube}"))?;
                let n = str::parse::<u64>(n).map_err(|_| format!("Invalid number: {n}"))?;
                let color = match cube {
                    "red" => Color::Red,
                    "green" => Color::Green,
                    "blue" => Color::Blue,
                    _ => Err(format!("Invalid color: {cube}"))?,
                };
                let max_for_color = maxes.entry(color).or_default();
                *max_for_color = (*max_for_color).max(n);
            }
        }
        Ok::<_, String>(acc + maxes.values().fold(1, |acc, max| acc * *max))
    })?;
    println!("{sum}");
    Ok(())
}

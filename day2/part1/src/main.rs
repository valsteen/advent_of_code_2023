use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{stdin, BufRead};

#[derive(Hash, Eq, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Color::Red => "red",
            Color::Green => "green",
            Color::Blue => "blue",
        }
        .fmt(f)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let constraints = HashMap::from([(Color::Red, 12), (Color::Green, 13), (Color::Blue, 14)]);
    let lines = stdin().lock().lines();
    let sum = lines
        .map_while(Result::ok)
        .enumerate()
        .try_fold(0, |acc, (line_number, line)| {
            let line_number = line_number + 1;
            let (_, line) = line.split_once(": ").ok_or_else(|| format!("Malformed line: {line}"))?;
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
                    let max = *constraints
                        .get(&color)
                        .ok_or_else(|| format!("missing constraint for {color}"))?;
                    if n > max {
                        return Ok::<_, String>(acc);
                    }
                }
            }
            Ok(acc + line_number)
        })?;
    println!("{sum}");
    Ok(())
}

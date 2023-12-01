use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let sum = lines
        .flatten()
        .try_fold(0, |acc, line| {
            let mut digits = line.chars().filter_map(|c| c.to_digit(10));
            let first = digits.next()?;
            Some(acc + first * 10 + digits.last().unwrap_or(first))
        })
        .ok_or("invalid input")?;

    println!("{sum}");
    Ok(())
}

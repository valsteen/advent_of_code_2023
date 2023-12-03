use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let sum = lines.map_while(Result::ok).try_fold(0, |acc, line| {
        let mut digits = line.chars().filter_map(|c| c.to_digit(10));
        let first = digits.next().ok_or(format!("No match in '{line}'"))?;
        Ok::<_, String>(acc + first * 10 + digits.last().unwrap_or(first))
    })?;

    println!("{sum}");
    Ok(())
}

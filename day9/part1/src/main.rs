use std::error::Error;
use std::io::{stdin, BufRead};

fn resolve(numbers: &[i64]) -> Option<i64> {
    Some(
        *numbers.last()?
            + match (0..numbers.len() - 1).fold(
                (true, Vec::with_capacity(numbers.len() - 1)),
                |(zeroes, mut acc), i| {
                    let result = numbers[i+1] - numbers[i];
                    let zeroes = zeroes && result == 0;
                    acc.push(result);
                    (zeroes, acc)
                },
            ) {
                (true, _) => 0,
                (_, next) => resolve(next.as_slice())?,
            },
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let sum = lines
        .map(|line| {
            let numbers = line?.split(' ').map(str::parse::<i64>).collect::<Result<Vec<_>, _>>()?;
            Ok(resolve(numbers.as_slice()).ok_or("cannot compute next number")?)
        })
        .sum::<Result<i64, Box<dyn Error>>>()?;

    println!("{sum}");
    Ok::<_, Box<dyn Error>>(())
}

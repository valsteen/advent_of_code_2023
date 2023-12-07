use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter;
#[derive(Debug)]
struct Race {
    time: u64,
    record: u64,
}

impl Race {
    fn runs(&self) -> usize {
        (0..=self.time)
            .filter(|time| (self.time - time) * time > self.record)
            .count()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut lines = lines.map_while(Result::ok);

    let times = lines
        .next()
        .ok_or("no times given")?
        .split_whitespace()
        .skip(1)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()?;
    let records = lines
        .next()
        .ok_or("no distances given")?
        .split_whitespace()
        .skip(1)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()?;

    let races = iter::zip(times, records)
        .map(|(time, record)| Race { time, record })
        .collect::<Vec<_>>();

    let possibilities = races.iter().map(Race::runs).product::<usize>();
    println!("{possibilities}");

    Ok::<_, Box<dyn Error>>(())
}
